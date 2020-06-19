use model::{self,Model,Instance,PreorderParams};
use precomputed::{self,Precomputed};
use std::result;
use std::fmt;
use std::convert::From;
use std::io::{Read,Write};
use rpc_common::{Subject,ChoiceRow};
use codec::{self,Encode,Decode,Packed};
use rayon::prelude::*;

pub type Result<T> = result::Result<T, EstimationError>;

#[derive(Debug)]
pub enum EstimationError {
    InstanceError(model::InstanceError),
    PreorderError(precomputed::Error),
}

impl Encode for EstimationError {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        match self {
            &EstimationError::InstanceError(ref e) => (0u8, e).encode(f),
            &EstimationError::PreorderError(ref e) => (1u8, e).encode(f),
        }
    }
}

impl fmt::Display for EstimationError {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        match self {
            &EstimationError::InstanceError(ref e) => e.fmt(f),
            &EstimationError::PreorderError(ref e) => e.fmt(f),
        }
    }
}

impl From<model::InstanceError> for EstimationError {
    fn from(e : model::InstanceError) -> EstimationError {
        EstimationError::InstanceError(e)
    }
}

impl From<precomputed::Error> for EstimationError {
    fn from(e : precomputed::Error) -> EstimationError {
        EstimationError::PreorderError(e)
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    subjects : Vec<Packed<Subject>>,
    models : Vec<model::Model>,
    fc : bool,
    disable_parallelism : bool,
}

impl Decode for Request {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Request> {
        Ok(Request {
            subjects: Decode::decode(f)?,
            models: Decode::decode(f)?,
            fc: Decode::decode(f)?,
            disable_parallelism: Decode::decode(f)?,
        })
    }
}

// fields public for testing
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct InstanceInfo {
    pub model : Model,
    pub entropy : f64,
    pub instance : Vec<u8>,
}

impl InstanceInfo {
    pub fn from(model : Model, entropy : f64, inst : &Instance) -> Self {
        InstanceInfo {
            model,
            entropy,
            instance: codec::encode_to_memory(inst).unwrap(),
        }
    }
}

impl Encode for InstanceInfo {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        self.model.encode(f)?;
        self.entropy.encode(f)?;
        self.instance.encode(f)
    }
}

pub struct Response {
    pub subject_name : String,
    pub minimal_entropy : f64,
    pub best_instances : Vec<InstanceInfo>,
}

impl Encode for Response {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (&self.subject_name, self.minimal_entropy, &self.best_instances).encode(f)
    }
}

struct BestInstances {
    lowest_entropy : Option<f64>,
    instances : Vec<InstanceInfo>,
}

impl BestInstances {
    fn new() -> Self {
        BestInstances {
            lowest_entropy: None,
            instances: Vec::new(),
        }
    }

    fn add_instance(&mut self, model : Model, entropy : f64, instance : Instance) {
        if let Some(ref mut lowest_entropy) = self.lowest_entropy {
            // HACK
            if (entropy - *lowest_entropy).abs() <= 1e-9 {
                // about the same entropy, include this
                self.instances.push(InstanceInfo::from(model, entropy, &instance));
            }
            else if entropy < *lowest_entropy {
                // strictly better
                self.instances.clear();
                self.instances.push(InstanceInfo::from(model, entropy, &instance));
                *lowest_entropy = entropy;
            } else {
                // we're strictly worse, forget this instance
            }
        } else {
            // no instances yet
            self.instances.clear();  // not necessary but let's do it anyway
            self.instances.push(InstanceInfo::from(model, entropy, &instance));
            self.lowest_entropy = Some(entropy);
        }
    }

    fn finish(self) -> Option<(Vec<InstanceInfo>, f64)> {
        if let Some(lowest_entropy) = self.lowest_entropy {
            Some((self.instances, lowest_entropy))
        } else {
            // no instances
            None
        }
    }

    fn combine(mut self, other : BestInstances) -> BestInstances {
        match (self.lowest_entropy, other.lowest_entropy) {
            (Some(se), Some(oe)) =>
                if (se - oe).abs() <= 1e-9 {
                    self.instances.extend(other.instances);
                    self
                } else if se < oe {
                    self
                } else {
                    other
                },

            (Some(_), None) => self,
            (None, Some(_)) => other,
            (None, None) => self,
        }
    }
}

fn evaluate_model(
    precomputed : &Precomputed,
    fc : bool,
    model : Model,
    alt_count : u32,
    choices : &[ChoiceRow],
) -> Result<BestInstances> {
    let mut model_instances = BestInstances::new();

    model::traverse_all(precomputed, model, alt_count, choices, &mut |inst| {
        model_instances.add_instance(
            model,
            inst.entropy(fc, choices),
            inst,
        )
    })?;

    Ok(model_instances)
}

pub fn run_one(precomputed : &Precomputed, fc : bool, subject : &Subject, models : &[Model]) -> Result<Response> {
    let alt_count = subject.alternatives.len() as u32;

    let mut best_instances = BestInstances::new();
    for &model in models {
        // do SRC only if UM/UC do not rationalise perfectly
        if model == Model::SequentiallyRationalizableChoice {
            continue;
        }

        best_instances = best_instances.combine(
            evaluate_model(precomputed, fc, model, alt_count, &subject.choices)?
        );
    }

    let (mut best_instances, minimal_entropy) = best_instances.finish().unwrap();
    best_instances.sort_unstable_by(|x,y| x.partial_cmp(y).unwrap());

    Ok(Response {
        subject_name: subject.name.clone(),
        best_instances,
        minimal_entropy,
    })
}

pub fn run(precomputed : &mut Precomputed, request : &Request) -> Result<Vec<Packed<Response>>> {
    // precompute up to the maximum number of alternatives
    let alt_count = request.subjects.iter().map(
        |subj| subj.unpack().alternatives.len() as u32
    ).max().expect("zero subjects in request");

    // don't precompute if searching only permutations (strict UM)
    if request.models != &[Model::PreorderMaximization(PreorderParams{strict:Some(true),total:Some(true)})] {
        precomputed.precompute(alt_count)?;
    }

    let results : Vec<Result<Response>> = if request.disable_parallelism {
        // run estimation sequentially
        request.subjects.iter().map(
            |subj| run_one(precomputed, request.fc, subj.unpack(), &request.models)
        ).collect()
    } else {
        // run estimation in parallel
        let mut results = Vec::new();
        request.subjects.par_iter().map(
            |subj| run_one(precomputed, request.fc, subj.unpack(), &request.models)
        ).collect_into_vec(&mut results);
        results
    };

    // collect results
    let mut responses = Vec::with_capacity(results.len());
    for result in results.into_iter() {
        responses.push(Packed(result?));
    }

    Ok(responses)
}

#[cfg(test)]
mod test {
    use precomputed::Precomputed;
    use model;
    use preorder;
    use fast_preorder;
    use base64;
    use model::{Instance,Penalty};
    use codec;
    use alt_set::AltSet;
    use alt::Alt;
    use rpc_common::{ChoiceRow,Subject};
    use std::iter::FromIterator;

    fn testsubj(alt_count : u32, choices : Vec<ChoiceRow>) -> Subject {
        Subject{
            name: String::from("subject"),
            alternatives: (0..alt_count).map(|s| s.to_string()).collect(),
            choices,
        }
    }

    #[test]
    fn top_two() {
        use model::Model;

        let choices = choices![
            [0,1,2,3] -> [0,1],
            [0,1,2] -> [0,1],
            [0,1,3] -> [0,1],
            [0,2,3] -> [0,2],
            [1,2,3] -> [1,2],
            [0,1] -> [0,1],
            [0,2] -> [0,2],
            [0,3] -> [0,3],
            [1,2] -> [1,2],
            [1,3] -> [1,3],
            [2,3] -> [2,3]
        ];

        let subject = testsubj(4, choices);
        let models = [Model::TopTwo];
        let mut precomputed = Precomputed::new(None);
        precomputed.precompute(4).unwrap();
        let response = super::run_one(&precomputed, &subject, &models).unwrap();

        assert_eq!(response.score, Penalty::exact(0));
        assert_eq!(response.best_instances.len(), 2);
    }

    #[test]
    fn undominated_detail() {
        let bytes = base64::decode("AgUBAgQPHwE=").unwrap();
        let inst : Instance = codec::decode_from_memory(&bytes).unwrap();

        let rows = choices![
            [0,1] -> [0,1],
            [0,1,3] -> [0,1],
            [0,1,4] -> [0,1],
            [1,2] -> [1,2],
            [1,2,3] -> [1,2],
            [1,2,4] -> [1,2],
            [0,2] -> [0,2],
            [0,2,3] -> [0,2],
            [0,2,4] -> [0,2],

            [0,3] -> [0],
            [1,3] -> [1],
            [2,3] -> [2],
            [0,4] -> [0],
            [1,4] -> [1],
            [2,4] -> [2],

            [3,4] -> [3]
        ];

        for cr in rows {
            assert_eq!(inst.choice(cr.menu.view(), None), cr.choice, "menu: {}", cr.menu);
        }
    }

    #[test]
    fn seqrc() {
        use model::Model;
        use super::InstanceInfo as II;

        let models = [Model::SequentiallyRationalizableChoice];
        let subject = testsubj(4, choices![
                [0,1,2,3] -> [1],
                [0,1,2] -> [1],
                [0,1,3] -> [1],
                [0,2,3] -> [0],
                [1,2,3] -> [2],
                [0,1] -> [1],
                [0,2] -> [0],
                [0,3] -> [0],
                [1,2] -> [2],
                [1,3] -> [1],
                [2,3] -> [2]
        ]);

        let mut precomputed = Precomputed::new(None);
        precomputed.precompute(4).unwrap();
        let response = super::run_one(&precomputed, &subject, &models).unwrap();

        assert_eq!(response.score, Penalty::exact(0));
        assert_eq!(response.best_instances.len(), 11);

        let model = Model::SequentiallyRationalizableChoice;
        let penalty = Penalty::exact(0);
        assert_eq!(response.best_instances, vec![
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 9, 4, 7, 6, 4, 14] },
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 11, 4, 7, 6, 4, 12] },
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 11, 4, 7, 6, 4, 14] },
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 11, 4, 15, 6, 4, 12] },
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 13, 4, 7, 6, 4, 14] },
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 15, 4, 7, 6, 4, 8] },
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 15, 4, 7, 6, 4, 12] },
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 15, 4, 7, 6, 4, 14] },
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 15, 4, 15, 6, 4, 8] },
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 15, 4, 15, 6, 4, 12] },
            II{ model, penalty: penalty.clone(), instance: vec![7, 4, 1, 2, 5, 15, 4, 15, 14, 4, 8] }
        ]);
    }

    #[test]
    fn undominated() {
        use model::Model;
        use super::InstanceInfo as II;

        let mut precomputed = Precomputed::new(None);
        precomputed.precompute(5).unwrap();

        let models = [Model::UndominatedChoice{strict: true}];
        let subject = testsubj(5, choices![
                [0,1] -> [0,1],
                [0,1,3] -> [0,1],
                [0,1,4] -> [0,1],
                [1,2] -> [1,2],
                [1,2,3] -> [1,2],
                [1,2,4] -> [1,2],
                [0,2] -> [0,2],
                [0,2,3] -> [0],
                [0,2,4] -> [2],
                
                [0,3] -> [0],
                [1,3] -> [1],
                [2,3] -> [2],
                [0,4] -> [0],
                [1,4] -> [1],
                [2,4] -> [2],

                [3,4] -> [3]
        ]);

        let response = super::run_one(&precomputed, &subject, &models).unwrap();
        assert_eq!(response.score, Penalty::exact(2));
        assert_eq!(response.best_instances.len(), 3);

        let m = Model::UndominatedChoice{strict: true};
        assert_eq!(response.best_instances, vec![
            II{ model: m, penalty: Penalty::exact(2), instance: vec![2, 5, 1, 2, 4, 15, 31] },
            II{ model: m, penalty: Penalty::exact(2), instance: vec![2, 5, 1, 2, 5, 15, 31] },
            II{ model: m, penalty: Penalty::exact(2), instance: vec![2, 5, 5, 2, 4, 15, 31] },
        ]);
    }

    #[test]
    fn indecisive() {
        use model::PreorderParams as PP;
        use model::Model::PreorderMaximization as PM;
        use alt_set::AltSet;

        let mut precomputed = Precomputed::new(None);
        precomputed.precompute(5).unwrap();

        let models = [PM(PP{ strict: None, total: None })];
        let subject = testsubj(5, choices![
                [0,1,2,3,4] -> [],
                [0,1,2,3] -> [],
                [0,1,2,4] -> [],
                [0,1,3,4] -> [],
                [0,2,3,4] -> [],
                [1,2,3,4] -> [],
                [0,1,2] -> [],
                [0,1,3] -> [],
                [0,2,3] -> [],
                [1,2,3] -> [],
                [0,1,4] -> [],
                [0,2,4] -> [],
                [1,2,4] -> [],
                [0,3,4] -> [],
                [1,3,4] -> [],
                [2,3,4] -> [],
                [3,4] -> [],
                [2,4] -> [],
                [1,4] -> [],
                [0,4] -> [],
                [2,3] -> [],
                [1,3] -> [],
                [0,3] -> [],
                [1,2] -> [],
                [0,2] -> [],
                [0,1] -> [],
                [4] -> [4],
                [3] -> [3],
                [2] -> [2],
                [1] -> [1],
                [0] -> [0]
        ]);

        // we want a 5-element diagonal here
        let p = preorder::Preorder::from_fast_preorder(5,
            fast_preorder::FastPreorder(0x1008040201)  // 8 bits per row
        );

        {
            let mut choice = alts![0,1];
            assert_eq!(choice, alts![0,1]);
            let up0 = p.upset(Alt(0));
            assert_eq!(up0.iter().collect::<AltSet>(), alts![0]);
            choice &= up0;
            assert_eq!(choice, alts![0]);
            let up1 = p.upset(Alt(1));
            assert_eq!(up1.iter().collect::<AltSet>(), alts![1]);
            choice &= up1;
            assert_eq!(choice, alts![]);
        }

        let instance = model::Instance::PreorderMaximization(p);
        assert_eq!(instance.choice(alts![0,1].view(), None), alts![]);

        let response = super::run_one(&precomputed, &subject, &models).unwrap();
        assert_eq!(response.score, Penalty::exact(0));
        assert_eq!(response.best_instances, vec![super::InstanceInfo{
            model: PM(PP{ strict: None, total: None }),
            penalty: Penalty::exact(0),
            instance: vec![0, 5, 1, 2, 4, 8, 16],
        }]);
    }
}
