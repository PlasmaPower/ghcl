use repository::{Repository, Service};

#[test]
fn from_arg_string() {
    assert_eq!(Repository::from_arg_string("foo/bar", Service::GitHub).expect("Failed to parse arg string"), Repository {
        service: Service::GitHub,
        user: "foo".into(),
        name: "bar".into(),
    }, "Failed to parse user/repo format arg string");

    assert_eq!(Repository::from_arg_string("https://github.com/foo/bar", Service::GitHub).expect("Failed to parse arg string"), Repository {
        service: Service::GitHub,
        user: "foo".into(),
        name: "bar".into(),
    }, "Failed to parse GitHub URL arg string");

    assert_eq!(Repository::from_arg_string("https://github.com/foo/bar/tree/branch?x=y#example", Service::GitHub).expect("Failed to parse arg string"), Repository {
        service: Service::GitHub,
        user: "foo".into(),
        name: "bar".into(),
    }, "Failed to parse detailed GitHub URL arg string");
}
