pub fn js_pre(string: &str) -> String {
    string.replace("'", "\\'")
        .replace("\\n", "\\\\n")
        .replace("\"", "\\\"")
}