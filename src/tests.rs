#[cfg(test)]
mod tests {
    #[test]
    fn duplicate_payloads_macro() {
        let t = trybuild::TestCases::new();
        t.compile_fail("compile_tests/multiple_payloads.rs");
    }

    #[test]
    fn normal_usage() {
        let t = trybuild::TestCases::new();
        t.pass("compile_tests/normal_usage.rs");
    }
}
