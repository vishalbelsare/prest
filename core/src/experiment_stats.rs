use std::result;
use std::io::{Read,Write};

use void::Void;
use codec::{Encode,Decode,Packed,self};
use rpc_common::{Subject};

type Error = Void;

type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Request {
    pub subject : Packed<Subject>,
}

impl Decode for Request {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Request> {
        Ok(Request{ subject: Decode::decode(f)? })
    }
}

// usize for convenience
pub struct Response {
    name : String,
    observations : usize,
    active_choices : usize,
    deferrals : usize,
}

impl Encode for Response {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (&self.name, self.observations, self.active_choices, self.deferrals).encode(f)
    }
}

pub fn run(request : Request) -> Result<Response> {
    let Request{ subject: Packed(Subject{name, alternatives:_, choices}) } = request;

    Ok(Response{
        name,
        observations: choices.len(),
        active_choices: choices.iter().filter(|cr| cr.choice.view().is_nonempty()).count(),
        deferrals: choices.iter().filter(|cr| cr.choice.view().is_empty()).count(),
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use alt_set::AltSet;
    use alt::Alt;
    use rpc_common::{Subject,ChoiceRow};
    use codec;
    use std::iter::FromIterator;

    fn testreq(alt_count : u32, choices : Vec<ChoiceRow>) -> Request {
        Request{subject: codec::Packed(Subject{
            name: String::from("subject"),
            alternatives: (0..alt_count).map(|s| s.to_string()).collect(),
            choices: choices,
        })}
    }

    #[test]
    fn basic() {
        let response = run(testreq(3, choices![
            [0,1,2] -> [2],
            [0,1] -> [],
            [0,2] -> [],
            [1,2] -> [2],
            [0] -> [0],
            [1] -> [],
            [2] -> [2]
        ])).unwrap();

        assert_eq!(response.observations, 7);
        assert_eq!(response.active_choices, 4);
        assert_eq!(response.deferrals, 3);
    }
}
