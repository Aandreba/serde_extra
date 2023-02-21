use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct MyVec<'a> (#[serde(borrow = "'a", with = "serde_extra::iter_map")] Vec<(i32, &'a str)>);


#[test]
fn vec_map () {
    let test = MyVec(vec![
        (1, "Hello"),
        (2, "World")
    ]);
    let json = serde_json::to_string_pretty(&test).unwrap();
    println!("{json}");
    let back: MyVec = serde_json::from_str(&json).unwrap();
    println!("{back:?}")
}