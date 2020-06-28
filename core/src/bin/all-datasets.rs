extern crate prest;
extern crate base64;

use prest::alt_set::{AltSet};
use prest::rpc_common::{Subject,ChoiceRow};
use prest::precomputed::Precomputed;
use prest::estimation;
use std::iter::FromIterator;

const NALT : u32 = 4;
const FC : bool = false;

fn fac(n : u64) -> u64 {
    (1..(n+1)).product()
}

fn comb(n : u64, k : u64) -> u64 {
    fac(n) / (fac(k) * fac(n-k))
}

fn datasets(n : u64) -> u64 {
    (1..(n+1)).map(|i|
        (if FC {i} else {i+1}).pow(comb(n, i) as u32)
    ).product()
}

fn main() {
    let alts = vec![
        String::from("a"),
        String::from("b"),
        String::from("c"),
        String::from("d")
    ];

    let precomputed = {
        let mut p = Precomputed::new(None);
        p.precompute(4).unwrap();
        p
    };

    let models = {
        use prest::model::*;
        use prest::model::Model::*;

        let pp_total = PreorderParams{strict: Some(true), total: Some(true)};
        let pp_non_total = PreorderParams{strict: Some(true), total: Some(false)};
        let pp_whatever = PreorderParams{strict: Some(true), total: None};

        if FC {
            vec![
                // UM
                PreorderMaximization(pp_total),

                // UMOO
                Unattractiveness{strict: true},

                // Overload
                Overload(pp_total),

                // SD
                SequentialDomination{strict: true},
            ]
        } else {
            vec![
                // UM
                PreorderMaximization(pp_total),

                // UMOO
                Unattractiveness{strict: true},

                // Overload
                Overload(pp_whatever),

                // MDC
                PreorderMaximization(pp_non_total),

                // PDC
                PartiallyDominantChoice{fc: false},

                // UC
                UndominatedChoice{strict: true},

                // SD
                SequentialDomination{strict: true},
            ]
        }
    };

    println!("dataset,entropy,model,instance");
    for code in 0..datasets(NALT as u64) {
        let mut j = code;
        let subject = Subject {
            name: code.to_string(),
            alternatives: alts.clone(),
            choices: AltSet::powerset(NALT).map(
                |menu| {
                    let n = menu.size();
                    let n_nfc = if FC { n } else { n + 1 };
                    let k = (j % n_nfc as u64) as u32;  // u64 mod u32 is safely u32
                    j = j / n_nfc as u64;

                    let choice = if k == n {
                        AltSet::empty()
                    } else {
                        AltSet::singleton(
                            Vec::from_iter(menu.view())[k as usize]
                        )
                    };

                    ChoiceRow {
                        menu,
                        default: None,
                        choice,
                    }
                }
            ).collect()
        };

        let response = estimation::run_one(
            &precomputed, true, &subject, &models
        ).unwrap();

        for instance in &response.best_instances {
            println!(
                "{},{},\"{:?}\",{}",
                code,
                instance.entropy,
                instance.model,
                base64::encode(&instance.instance),
            );
        }
    }
}
