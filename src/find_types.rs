use std::collections::HashSet;
use std::error::Error;
use std::fs::{self, read_to_string};
use std::path::Path;

use colored::Colorize;

pub fn handle_find_types_command() {
    let res = try_find_types();
    match res {
        Ok(_) => (),
        Err(e) => {
            println!("Error trying to find all types: {}", e);
        }
    }
}

fn try_find_types() -> Result<(), Box<dyn Error>> {
    let dir = Path::new("resources/metadata/");

    let mut files_content = Vec::new();

    for entry_result in fs::read_dir(dir)? {
        let entry = entry_result?;
        let path = entry.path();

        if path.is_file() {
            let content = read_to_string(&path)?;
            files_content.push((path, content));
        }
    }
    let problems_amt = files_content.len();

    let type_pattern = r#""type": ""#; // "type": "
    let mut found_types: HashSet<String> = HashSet::new();
    let mut cpt = 0;
    for content in files_content {
        let lines = content.1.lines();
        for l in lines {
            if l.contains(type_pattern) {
                cpt += 1;
                let components = l.split("\"").collect::<Vec<&str>>();
                found_types.insert(components[components.len() - 2].into());
            }
        }
    }

    println!(
        "Found {} different types among {} -> {:?}\n",
        found_types.len(),
        cpt,
        found_types
    );

    compute_coupon_probability(found_types, problems_amt);

    Ok(())
}

fn compute_coupon_probability(found_types: HashSet<String>, problems_amt: usize) {
    let found_types = get_true_types(found_types);
    println!(
        "Found {} true different types -> {:?}\n",
        found_types.len(),
        found_types
    );

    // P(T>=cnH_n) <= 1/c
    let n_p1 = found_types.len() + 1;
    let n_hn_p1 = n_p1 as f32 * nth_harmonic(n_p1);
    let c = problems_amt as f32 / n_hn_p1;
    let inv_c = 1. / c;
    let inv_c_pp = (10000. * inv_c).floor() / 100.;
    println!("Assuming that there is another type (and assuming equiprobable distribution), the probability that we would not have found it by now is below {}{}", inv_c_pp.to_string().bold().cyan(), "%".bold().cyan());

    let t_5_percent = 20. * n_hn_p1;
    println!(
        "For 5% you should test {} files. You tested {}",
        t_5_percent.floor(),
        problems_amt
    );
}

fn nth_harmonic(n: usize) -> f32 {
    let mut sum = 0.;
    for i in 1..=n {
        sum += 1. / i as f32;
    }

    sum
}

fn get_true_types(found_types: HashSet<String>) -> HashSet<String> {
    let modifiers: [&str; 2] = ["[]", "list"];

    let mut true_types = HashSet::new();
    for t in found_types {
        let mut is_modified = false;
        for pattern in modifiers {
            if t.contains(pattern) {
                is_modified = true;
                break;
            }
        }
        if !is_modified {
            true_types.insert(t.clone());
        }
    }
    true_types
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_true_types() {
        let mut found_types: HashSet<String> = HashSet::new();
        found_types.insert("foo".into());
        found_types.insert("bar".into());
        found_types.insert("not[]".into());
        found_types.insert("list<meow>".into());
        let res = get_true_types(found_types);
        let mut expected: HashSet<String> = HashSet::new();
        expected.insert("foo".into());
        expected.insert("bar".into());
        assert_eq!(res, expected);
    }
}
