use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::json;


#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
pub(crate) struct MagicParam {
    indir: i32,
    name: i32,
    elf_phnum: i32,
    elf_shnum: i32,
    elf_notes: i32,
    regex: i32,
    bytes: i32,
}


impl MagicParam {
    pub(crate) fn from_cmd_line(line: &str) -> MagicParam {
        let default = MagicParam::default();

        let mut mp: serde_json::Value = serde_json::to_value(&default).unwrap();
        for field_assign in line.split(";") {
            let field_assign: Vec<&str> = field_assign.split("=").collect();
            if field_assign.len() != 2 {
                panic!("Each assignment should consist of exactly two parts separated by `='")
            }

            let field = field_assign[0];
            if mp[field] == json!(null) {
                panic!("Unknown field `{:?}' for :class MagicParam!", field);
            }

            match field_assign[1].parse::<i32>() {
                Ok(assign) => {
                    mp[field] = json!(assign);
                },
                Err(_) => {
                    panic!("The assigned value to the field should be a number!");
                },
            }
        }
        serde_json::from_value(mp).unwrap()
    }
}


#[cfg(test)]
mod tests {
    use crate::magic_param::MagicParam;

    #[test]
    fn test_from_cmd_line() {
        let input = "name=10;regex=12";

        let get = MagicParam::from_cmd_line(input);
        let expect = MagicParam {
            name: 10,
            regex: 12,
            ..MagicParam::default()
        };
        assert_eq!(get, expect);
    }
}
