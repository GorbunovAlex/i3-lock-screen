use pam::Authenticator;

pub fn authenticate(user: &str, password: &str) -> bool {
    let mut auth = Authenticator::with_password("login").expect("PAM init failed");
    auth.get_handler().set_credentials(user, password);
    auth.authenticate().is_ok()
}
