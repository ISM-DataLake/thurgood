
pub mod consts;
pub mod error;
mod rb_type;
pub use rb_type::RbType;
pub use error::{ThurgoodError, TResult};

// Uncomment to get IDE help, re-comment for build/release/push.
// pub use std::rc::Rc as RcType; pub mod inner;

pub mod rc {
    pub use std::rc::Rc as RcType;
    #[path="../inner/mod.rs"]
    mod inner;
    pub use inner::*;
}

pub mod arc {
    pub use std::sync::Arc as RcType;
    #[path="../inner/mod.rs"]
    mod inner;
    pub use inner::*;
}

#[cfg(test)]
mod tests {
    use std::io;

    use crate::rc::*;
    // use crate::inner::*;

    /// Parse a string into an `RbAny`
    fn reader_parse(s: &str) -> RbAny {
        RbReader::new(io::Cursor::new(s.as_bytes())).read().expect("Parsing error")
    }

    /// Writes `value` to a `Vec<u8>` and returns it.
    fn writer_write(value: &RbAny) -> Vec<u8> {
        let mut buf = Vec::new();
        RbWriter::new(&mut buf).write(value).expect("Writing error");
        buf
    }

    #[test]
    fn array_string_hash() {
        let inp = "\x04\x08[\x07I\"\ttest\x06:\x06ET{\x06:\x06aI\"\x06b\x06;\x00T";
        let exp = RbAny::from(vec![
                RbAny::from("test"),
                RbAny::from(RbHash::from_pairs(vec![
                    ( RbSymbol::from("a").into(), RbAny::from("b") )
                ])),
            ]);
        assert_eq!(reader_parse(inp), exp);
        assert_eq!(writer_write(&exp).as_slice(), inp.as_bytes());
    }

    #[test]
    fn class_and_int() {
        let inp = "\x04\x08[\x07o:\x08Foo\x07:\n@nameI\"\tJack\x06:\x06ET:\t@agei\x1Eo;\x00\x07;\x06I\"\tJane\x06;\x07T;\x08i\x1D";
        let sym_name = RbSymbol::from("@name");
        let sym_age = RbSymbol::from("@age");
        let exp = RbAny::from(vec![
            RbObject::new_from_slice("Foo", &vec![
                ("@name", "Jack".into()),
                ("@age", 25.into()),
            ]).into_object().into(),
            RbRef::new_object("Foo", &vec![
                (sym_name.clone(), "Jane".into()),
                (sym_age.clone(), 24.into()),
            ]).into_any(),
        ]);
        assert_eq!(reader_parse(inp), exp);
        assert_eq!(writer_write(&exp).as_slice(), inp.as_bytes());
    }

    #[test]
    fn modules() {
        let inp = "\x04\x08{\x07:\x07aao:\x0EBar::BazA\x00:\x07bbo:\x0EBar::BazB\x00";
        let sym_aa = RbSymbol::from("aa");
        let sym_bar_baz_a = RbSymbol::from("Bar::BazA");
        let sym_bb = RbSymbol::from("bb");
        let sym_bar_baz_b = RbSymbol::from("Bar::BazB");
        let exp = RbHash::from_pairs(vec![
                (sym_aa.clone().into(), RbRef::new_object(&sym_bar_baz_a, &vec![]).into()),
                (sym_bb.clone().into(), RbRef::new_object(&sym_bar_baz_b, &vec![]).into()),
            ]).into();
        assert_eq!(reader_parse(inp), exp);
        assert_eq!(writer_write(&exp).as_slice(), inp.as_bytes());
    }

    #[test]
    fn object_ref_count_1() {
        let inp = "\x04\x08[\no:\x08Foo\x07:\n@nameI\"\tJack\x06:\x06ET:\t@agei\x1E@\x06{\x06:\x08key@\x06o;\x00\x07;\x06I\"\tJane\x06;\x07T;\x08i\x1D@\t";
        let sym_name: RbSymbol = "@name".into();
        let sym_age: RbSymbol = "@age".into();
        let sym_key: RbSymbol = "key".into();
        let ob_1 = RbRef::new_object("Foo", &vec![
            (sym_name.clone(), RbAny::from("Jack") ),
            (sym_age.clone(), RbAny::Int(25) ),
            ]).into_any();
        let ob_2 = RbRef::new_object("Foo", &vec![
            (sym_name.clone(), RbAny::from("Jane") ),
            (sym_age.clone(), RbAny::Int(24) ),
            ]).into_any();
        let exp = RbRef::Array(vec![
                ob_1.clone(),
                ob_1.clone(),
                RbHash::from_pairs(vec![
                    (sym_key.as_any(), ob_1.clone() )
                ]).into(),
                ob_2.clone(),
                ob_2.clone(),
            ]).into_any();
        assert_eq!(reader_parse(inp), exp);
        assert_eq!(writer_write(&exp).as_slice(), inp.as_bytes());
    }

    #[test]
    fn object_ref_count_2() {
        let inp = "\x04\x08[\x07[\x06I\"\tTest\x06:\x06ET@\x06";
        let ob_1 = RbRef::Array(vec![ RbAny::from("Test") ]).into_any();
        let exp = RbRef::Array(vec![
            ob_1.clone(),
            ob_1.clone(),
        ]).into_any();
        assert_eq!(reader_parse(inp), exp);
        assert_eq!(writer_write(&exp).as_slice(), inp.as_bytes());
    }

    #[test]
    fn read_extended() {
        let inp = "\x04\x08e:\x08Bar[\x00";
        let sym_bar = RbSymbol::from("Bar");
        let exp = RbRef::Extended {
            module: sym_bar.clone(),
            object: RbRef::Array(vec![]).into(),
        }.into_any();
        assert_eq!(reader_parse(inp), exp);
        assert_eq!(writer_write(&exp).as_slice(), inp.as_bytes());
    }

    /// Technically floats can be stored as NULL-terminated C-strings. This is dumb, but here's
    /// a test for it anyways. This also tests normal floats.
    #[test]
    fn float_types() {
        let inp = "\x04\x08[\x07f\x0D0.123\x00NOf\n1.234";
        let out = "\x04\x08[\x07f\n0.123f\n1.234";
        let exp = RbAny::from(vec![
            RbAny::from(0.123f32),
            RbAny::from(1.234f32),
        ]);
        assert_eq!(reader_parse(inp), exp);
        assert_eq!(writer_write(&exp).as_slice(), out.as_bytes());
    }

}