#[cfg(test)]
mod tests {
    #[test]
    fn showcase_example() {
        let t = trybuild::TestCases::new();
        t.compile_fail("compile_tests/multiple_payloads.rs");
    }
}
