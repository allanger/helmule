use chrono::prelude::*;
use handlebars::{handlebars_helper, Handlebars};

handlebars_helper!(date_helper: | | Utc::now().format("%Y-%m-%d").to_string());
handlebars_helper!(time_helper: | | Utc::now().format("%H-%M-%S").to_string());

pub(crate) fn register_handlebars() -> Handlebars<'static> {
    let mut handlebars = Handlebars::new();
    handlebars.register_helper("date", Box::new(date_helper));
    handlebars.register_helper("time", Box::new(time_helper));
    handlebars
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::Utc;

    use crate::template::register_handlebars;

    #[test]
    fn test_date_helper() {
        let mut reg = register_handlebars();
        reg.register_template_string("test", "{{ date }}").unwrap();
        let result = reg
            .render("test", &HashMap::<String, String>::new())
            .unwrap();
        let expected = Utc::now().format("%Y-%m-%d").to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_time_helper() {
        let mut reg = register_handlebars();
        reg.register_template_string("test", "{{ time }}").unwrap();
        let result = reg
            .render("test", &HashMap::<String, String>::new())
            .unwrap();
        let expected = Utc::now().format("%H-%M-%S").to_string();
        assert_eq!(result, expected);
    }
}
