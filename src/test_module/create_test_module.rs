use crate::{
    logger::Log,
    parse_api::{HasSpecialDataType, ProbMetaData},
    read_write,
    test_module::{class_problem, function_problem},
    LOGGER,
};

pub const TEST_FUNCTION_ID_PATTERN: &str = "%EXAMPLE_ID%";
pub const TEST_FUNCTION_CONTENT_PATTERN: &str = "%CONTENT%";
pub const TEST_MODULE_FUNCTIONS_PATTERN: &str = "%TEST_FUNCTIONS%";
pub const TEST_MODULE_ADDITIONAL_IMPORTS_PATTERN: &str = "%ADDITIONAL_IMPORTS%";

pub fn try_create_test_module(
    example_testcases: &str,
    metadata: &ProbMetaData,
) -> Result<String, ()> {
    LOGGER.log("Trying to generate test module...");
    // read the mold for the test functions
    let function_mold = read_write::try_read_test_function_mold().map_err(|e| {
        println!("{e}");
    })?;

    // read the mold for the test module
    let module_mold = read_write::try_read_test_module_mold().map_err(|e| {
        println!("{e}");
    })?;

    // create the test functions
    let test_functions = match metadata {
        ProbMetaData::Class(metadata) => {
            class_problem::try_create_test_functions(&function_mold, example_testcases, metadata)
        }
        ProbMetaData::Function(metadata) => {
            function_problem::try_create_test_functions(&function_mold, example_testcases, metadata)
        }
    }
    .map_err(|e| {
        println!("{e}");
    })?;

    // add eventual imports
    let mut additional_imports_replacement = String::new();
    if metadata.has_tree_node() {
        additional_imports_replacement.push_str("\n    use crate::tree;");
    }
    if metadata.has_list_node() {
        additional_imports_replacement.push_str("\n    use crate::linked;");
    }

    let module_mold = module_mold.replace(
        TEST_MODULE_ADDITIONAL_IMPORTS_PATTERN,
        &additional_imports_replacement,
    );

    Ok(module_mold.replace(TEST_MODULE_FUNCTIONS_PATTERN, &test_functions))
}

#[cfg(test)]
mod tests {
    use crate::parse_api::{
        ClassMetaData, ConstructorJson, DataType, FunctionMetaData, ParamJson, ReturnJson,
        ScalarType,
    };

    use super::*;

    #[test]
    fn test_create_test_module_function_problem() {
        let example_testcases = r"[4,5,6,7,0,1,2]
0";
        let params: Vec<ParamJson> = vec![
            ParamJson {
                name: "nums".into(),
                _type: DataType {
                    scalar_type: ScalarType::Integer,
                    vec_depth: 1,
                },
            },
            ParamJson::from_scalar("target", ScalarType::Integer),
        ];
        let metadata = ProbMetaData::Function(FunctionMetaData {
            name: "search".into(),
            params: params.clone(),
            _return: Some(ReturnJson {
                _type: DataType::from(ScalarType::Integer),
            }),
        });
        let res = try_create_test_module(example_testcases, &metadata);
        let expected = r"

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_1() {
        let nums: Vec<i32> = vec![4,5,6,7,0,1,2];
        let target: i32 = 0;
        let res = Solution::search(nums, target);
        let expected: i32 = todo!(); // Fill in this value
        assert_eq!(res, expected);
    }
}"
        .into();

        if cfg!(windows) {
            assert_eq!(res.map(|s| s.replace('\r', "")), Ok(expected));
        } else {
            assert_eq!(res, Ok(expected));
        }
    }

    #[test]
    fn test_create_test_module_with_tree_node_function_problem() {
        let example_testcases = r"[1,null,2]";
        let params = vec![ParamJson::from_scalar("tree", ScalarType::TreeNode)];
        let metadata = ProbMetaData::Function(FunctionMetaData {
            name: "hug".into(),
            params: params.clone(),
            _return: Some(ReturnJson {
                _type: DataType::from(ScalarType::Integer),
            }),
        });
        let res = try_create_test_module(example_testcases, &metadata);
        let expected = r"

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree;

    #[test]
    fn example_1() {
        let tree: Option<Rc<RefCell<TreeNode>>> = tree![1,null,2];
        let res = Solution::hug(tree);
        let expected: i32 = todo!(); // Fill in this value
        assert_eq!(res, expected);
    }
}"
        .into();

        if cfg!(windows) {
            assert_eq!(res.map(|s| s.replace('\r', "")), Ok(expected));
        } else {
            assert_eq!(res, Ok(expected));
        }
    }

    #[test]
    fn test_create_test_module_class_problem() {
        let example_testcases = r#"["LRUCache","put","put","get"]
[[2],[1,1],[2,2],[1]]"#;
        let metadata = ProbMetaData::Class(ClassMetaData {
            class_name: "LRUCache".into(),
            constructor: ConstructorJson {
                params: vec![ParamJson::from_scalar("capacity", ScalarType::Integer)],
            },
            methods: vec![
                FunctionMetaData {
                    name: "get".into(),
                    params: vec![ParamJson::simple("key")],
                    _return: Some(ReturnJson {
                        _type: DataType::from(ScalarType::Integer),
                    }),
                },
                FunctionMetaData {
                    name: "put".into(),
                    params: vec![ParamJson::simple("key"), ParamJson::simple("value")],
                    _return: None,
                },
            ],
            _return: ReturnJson {
                _type: DataType::from(ScalarType::String),
            },
        });
        let res = try_create_test_module(example_testcases, &metadata);
        let expected = r"

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_1() {
        let capacity: i32 = 2;
        let my_class: LRUCache = LRUCache::new(capacity);
        let key: i32 = 1;
        let value: i32 = 1;
        my_class.put(key, value);
        let key: i32 = 2;
        let value: i32 = 2;
        my_class.put(key, value);
        let key: i32 = 1;
        let res = my_class.get(key);
        let expected: i32 = todo!(); // Fill in this value
        assert_eq!(res, expected);
    }
}"
        .into();

        if cfg!(windows) {
            assert_eq!(res.map(|s| s.replace('\r', "")), Ok(expected));
        } else {
            assert_eq!(res, Ok(expected));
        }
    }
}
