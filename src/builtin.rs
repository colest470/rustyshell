#[cfg(test)]
mod tests {
    use crate::helpers::split_redirection_args;

    #[test]
    fn cat_redirect_keeps_first_file_and_command_in_args() {
        let tokens = vec![
            "cat".to_string(),
            "/tmp/bee/orange".to_string(),
            "nonexistent".to_string(),
            "1>".to_string(),
            "/tmp/ant/owl.md".to_string(),
        ];

        let (args, stdout_redirect, stderr_redirect, append_redirect) = split_redirection_args(&tokens);

        assert_eq!(args, vec!["cat", "/tmp/bee/orange", "nonexistent"]);
        assert_eq!(stdout_redirect, Some("/tmp/ant/owl.md"));
        assert_eq!(stderr_redirect, None);
        assert_eq!(append_redirect, None);
    }

    #[test]
    fn append_redirect_is_parsed_as_a_single_operator() {
        let tokens = vec![
            "ls".to_string(),
            "-1".to_string(),
            "/tmp/ant".to_string(),
            ">>".to_string(),
            "/tmp/dog/bee.md".to_string(),
        ];

        let (args, stdout_redirect, stderr_redirect, append_redirect) = split_redirection_args(&tokens);

        assert_eq!(args, vec!["ls", "-1", "/tmp/ant"]);
        assert_eq!(stdout_redirect, None);
        assert_eq!(stderr_redirect, None);
        assert_eq!(append_redirect, Some("/tmp/dog/bee.md"));
    }
}