use crate::model;
extern crate diesel;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use crate::users;

//use diesel::dsl::count_star;


/// Run query using Diesel to insert a new database row and return the result.
pub fn find_user_by_uid(
    uid: i32,
    user_service: &dyn users::UserServiceTrait,
) -> Result<model::Response<model::User>, diesel::result::Error> {
    let user = user_service.get_user_by_uid(uid);

    if let Err(e) = user{
        return Err(e);
    }

    let user = user.unwrap();

    match user {
        Some(u) => return Ok(
            model::Response{
                success: true,
                data: Some(u),
                message: format!("User found!"),
                code: 200
            }
        ),
        None => return Ok(model::Response{
                success: false,
                data: None,
                message: format!("User not found!"),
                code: 404
            }
        )
    }
}

pub fn login(
    data: model::Login,
    user_service: &dyn users::UserServiceTrait,
) -> Result<model::Response<String>, diesel::result::Error> {
    let email = data.email.clone();
    let user = user_service.get_user_by_email(email.clone());

    match user {
        Ok(usr) => {
            if let Some(usr) = usr {
                let mut hasher = Sha256::new();
                hasher.input_str(&data.pass);
                let passwd = hasher.result_str();

                if passwd == usr.pass{
                    let now = SystemTime::now();
                    hasher = Sha256::new();
                    let dur = now.duration_since(UNIX_EPOCH)
                        .expect("Time went backwards");
                    hasher.input(format!("{}",dur.as_secs()).as_bytes());

                    let result = hasher.result_str();
                    let res = user_service.insert_token(email.clone(), result);

                    match res {
                        Ok(v) => {
                            if let Some(v) = v{
                                return Ok(
                                    model::Response{
                                        success: true,
                                        data: Some(format!("{}", v)),
                                        message: format!("Successfully logged in!"),
                                        code: 200
                                    }
                                );
                            } else{
                                return Ok(
                                    model::Response{
                                        success: false,
                                        data: None,
                                        message: format!("Error inserting token!"),
                                        code: 500
                                    }
                                );
                            }
                        }
                        ,
                        Err(e) => return Err(e)
                    }
                }else{
                    return Ok(model::Response{
                                success: false,
                                data: None,
                                message: format!("Wrong credentials!"),
                                code: 401
                            }
                        );
                }
            } else {
                return Ok(model::Response{
                            success: false,
                            data: None,
                            message: format!("User not found!"),
                            code: 400
                        }
                    );
            }
        },
        Err(e) => return Err(e)
    }
}

pub fn logout(
    data: model::TokenInfo,
    user_service: &dyn users::UserServiceTrait,
) -> Result<model::Response<bool>, diesel::result::Error> {

    let email = data.email.clone();
    let token = data.token.clone();
    let user = user_service.get_user_by_email(email.clone());
    match user {
        Ok(usr) => {
            if let Some(_) = usr {
                let res = user_service.remove_token(email.clone(), token);

                match res {
                    Ok(r) => if r {
                        return Ok(
                            model::Response{
                                success: true,
                                data: Some(true),
                                message: format!("OK"),
                                code: 200
                            });
                        }else{
                            return Ok(model::Response{
                                    success: true,
                                    data: Some(false),
                                    message: format!("Token not found!"),
                                    code: 400
                                }
                            );
                        },
                    Err(e) => return Err(e)
                }
            } else {
                return Ok(model::Response{
                            success: false,
                            data: None,
                            message: format!("User not found!"),
                            code: 400
                        }
                    );
            }
        },
        Err(e) => return Err(e)
    }
}

pub fn verify_token(
    data: model::TokenInfo,
    user_service: &dyn users::UserServiceTrait,
) -> Result<model::Response<bool>, diesel::result::Error> {
    let email = data.email.clone();
    let token = data.token.clone();
    let user = user_service.get_user_by_email(email);
    match user {
        Ok(usr) => {
            if let Some(usr) = usr {
                if let Some(tokens) = usr.tokens {

                    if let Some(_) = tokens.iter().position(|x| *x == token) {
                        return Ok(
                            model::Response{
                                success: true,
                                data: Some(true),
                                message: format!("OK"),
                                code: 200
                            }
                        );
                    }else{
                        return Ok(model::Response{
                                success: true,
                                data: Some(false),
                                message: format!("Unauthorized"),
                                code: 401
                            }
                        );
                    }
                } else{
                    return Ok(model::Response{
                                success: true,
                                data: Some(false),
                                message: format!("User has no tokens!"),
                                code: 401
                            }
                        );
                }
            } else {
                return Ok(model::Response{
                            success: false,
                            data: None,
                            message: format!("User not found!"),
                            code: 400
                        }
                    );
            }
        },
        Err(e) => return Err(e)
    }
}


pub fn register(
    mut data: model::RegisterUser,
    user_service: &dyn users::UserServiceTrait,
) -> Result<model::Response<model::User>, diesel::result::Error> {
    let email = data.email.clone();

    let mut hasher = Sha256::new();
    hasher.input_str(&data.pass);

    data.pass = hasher.result_str();

    let found = user_service.get_user_by_email(email.clone());

    if let Ok(Some(_)) = found{
        return Ok(
            model::Response{
                success: false,
                message: format!("User already exists!"),
                data: None,
                code: 400
            }
        ); 
    }


    let rows_affected = user_service.register_user(data);

    match rows_affected {
        Ok(rows) => {
            if rows == 1 {
                let user = user_service.get_user_by_email(email);
                match user {
                    Ok(usr) => return Ok(
                        model::Response{
                            success: true,
                            message: format!("Successfully registered user"),
                            data: usr,
                            code: 200
                        }
                    ),
                    Err(_) => return Ok(
                        model::Response{
                            success: false,
                            message: format!("Error registering user!"),
                            data: None,
                            code: 500
                        }
                    )
                }
            }else{
                return Ok(
                        model::Response{
                            success: false,
                            message: format!("User already exists!"),
                            data: None,
                            code: 400
                        }
                    );
            }
        },
        Err(e) => return Err(e)
    }
}


pub fn update(
    data: model::UpdateRequest,
    user_service: &dyn users::UserServiceTrait,
) -> Result<model::Response<model::User>, diesel::result::Error> {
    let is_logged = verify_token(model::TokenInfo{
            email: data.email.clone(),
            token: data.token.clone()
        }, user_service);

    if let Ok(is_logged) = is_logged {
        if is_logged.success && is_logged.code == 200 {

            let mut usr_data = data.user_data;
            let mut hasher = Sha256::new();
            hasher.input_str(&usr_data.pass);
            let passwd = hasher.result_str();
            usr_data.pass = passwd;
            let new_mail = usr_data.email.clone();
            let rows_affected = user_service.update_user(data.email.clone(), usr_data);
            match rows_affected {
                Ok(v) => {
                    if v == 1 {
                        let updated_usr = user_service.get_user_by_email(new_mail);
                        return Ok(
                            model::Response{
                                success: true,
                                message: format!("Successfully updated user!"),
                                data: updated_usr.unwrap(),
                                code: 200
                            }
                        )
                    }else{
                        return Ok(
                            model::Response{
                                success: false,
                                message: format!("User not found."),
                                data: None,
                                code: 400
                            }
                        )
                    }
                },
                Err(e) => return Err(e)
            }
        } else {
            return Ok(
                model::Response{
                    success: is_logged.success,
                    message: is_logged.message,
                    data: None,
                    code: is_logged.code
                }
            )
        }
    } else {
        return Ok(
            model::Response{
                success: false,
                message: format!("Internal error!"),
                data: None,
                code: 500
            }
        )
    }

}

#[cfg(test)]
mod tests {
    use crate::users;
    use crate::actions;
    use crate::model;
    #[test]
    fn find_user_by_id() {
        let u = users::UserServiceTest{};
        
        let res = actions::find_user_by_uid(1, &u);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.success);
        assert!(res.data.is_some());
        let data = res.data.unwrap();
        assert!(data.email == "test@email.com");
    }

    #[test]
    fn login_succ() {
        let u = users::UserServiceTest{};
        let req = model::Login{
            email: format!("test@email.com"),
            pass: format!("123456")
        };
        let res = actions::login(req, &u);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.success);
        assert!(res.data.is_some());
        let data = res.data.unwrap();
        assert!(data.len() > 0);
    }

    #[test]
    fn login_fail() {
        let u = users::UserServiceTest{};
        let req = model::Login{
            email: format!("test@email.com"),
            pass: format!("1234567")
        };
        let res = actions::login(req, &u);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(!res.success);
        assert!(res.data.is_none());
    }

    #[test]
    fn register_fail() {
        let u = users::UserServiceTest{};
        let req = model::RegisterUser{
            email: format!("test@email.com"),
            pass: format!("1234567"),
            firstname: format!("Test"),
            lastname: format!("Test")
        };
        let res = actions::register(req, &u);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(!res.success);
        assert!(res.data.is_none());
    }

    #[test]
    fn logout_fail() {
        let u = users::UserServiceTest{};
        let req = model::TokenInfo{
            email: format!("test@email.com"),
            token: format!("1234567")
        };
        let res = actions::logout(req, &u);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.success);
        assert!(!res.data.is_none());
        assert!(!res.data.unwrap());
    }

    #[test]
    fn logout_not_found() {
        let u = users::UserServiceTest{};
        let req = model::TokenInfo{
            email: format!("test2@email.com"),
            token: format!("1234567")
        };
        let res = actions::logout(req, &u);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(!res.success);
        assert!(res.data.is_none());
    }

    #[test]
    fn verify_fail() {
        let u = users::UserServiceTest{};
        let req = model::TokenInfo{
            email: format!("test2@email.com"),
            token: format!("1234567")
        };
        let res = actions::verify_token(req, &u);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(!res.success);
        assert!(res.data.is_none());
    }


    #[test]
    fn update_fail() {
        let u = users::UserServiceTest{};
        let reg = model::RegisterUser{
            firstname: format!("Test"),
            lastname: format!("Test"),
            email: format!("test@email.com"),
            pass: format!("123456")
        };

        let req = model::UpdateRequest{
            email: format!("test@email.com"),
            token: format!("tok"),
            user_data: reg
        };

        let res = actions::update(req, &u);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.success);
        println!("{}",res.code);
        assert!(res.data.is_some());
    }
}