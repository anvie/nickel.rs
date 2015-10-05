#[macro_use] extern crate nickel;
use nickel::{Nickel, HttpRouter};
use std::collections::HashMap;
//use std::io::Read;

use nickel::BodyFormDataString;

fn main() {
    let mut server = Nickel::new();

    server.get("/", middleware! { |_, res|
        let mut data = HashMap::new();
        data.insert("title","Contact");

        return res.render("examples/form_data/views/contact.html", &data)
    });

    server.post("/confirmation", middleware!{ |req, res|
        // let mut form_data = String::new();
        // req.origin.read_to_string(&mut form_data).unwrap();
        //
        // println!("{}", form_data);
        //
        let mut data = HashMap::new();
        data.insert("title", "Confirmation");
        // data.insert("formData", &form_data);
        //
        //
        let form_data = req.form_data();
        let firstname = form_data.get("firstname").unwrap();
        let lastname = form_data.get("lastname").unwrap();
        let phone = form_data.get("phone").unwrap();
        let email = form_data.get("email").unwrap();

        data.insert("firstname", firstname);
        data.insert("lastname", lastname);
        data.insert("phone", phone);
        data.insert("email", email);

        return res.render("examples/form_data/views/confirmation.html", &data)
    });

    server.listen("0.0.0.0:8080");
}
