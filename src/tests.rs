#[cfg(test)]
mod tests {
    #[test]
    fn normal_usage() {
        let t = trybuild::TestCases::new();
        t.pass("compile_tests/normal_usage.rs");
    }

    #[test]
    fn duplicate_payloads_macro() {
        let t = trybuild::TestCases::new();
        t.compile_fail("compile_tests/multiple_payloads.rs");
    }

    #[test]
    fn no_addresses() {
        let t = trybuild::TestCases::new();
        t.compile_fail("compile_tests/no_addresses.rs");
    }

    #[test]
    fn non_enum_address() {
        let t = trybuild::TestCases::new();
        t.compile_fail("compile_tests/non_enum_address.rs");
    }
}
