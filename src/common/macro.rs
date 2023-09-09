#[macro_export]
macro_rules! struct_from_json {
    ($s: ident, $filename: expr) => {
        let f = File::open(format!("./tests/assets/{}.json", $filename)).expect("File not found");
        let v = serde_json::from_reader::<File, Value>(f).expect("Invalid JSON data");

        $s::deserialise(v).unwrap();
    };
}
