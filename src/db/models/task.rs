use gettextrs::gettext;
use gtk::glib::FromVariant;
use gtk::{glib, glib::Properties, prelude::*, subclass::prelude::*};
use rusqlite;
use std::cell::{Cell, RefCell};
use std::fmt::Display;

use crate::db::models::Record;
use crate::db::operations::{read_records, read_tasks};

mod imp {
    use super::*;

    #[derive(Default, Debug, Properties)]
    #[properties(wrapper_type=super::Task)]
    pub struct Task {
        #[property(get, set)]
        pub id: Cell<i64>,
        #[property(get, set)]
        pub name: RefCell<String>,
        #[property(get, set)]
        pub done: Cell<bool>,
        #[property(get, set)]
        pub project: Cell<i64>,
        #[property(get, set)]
        pub section: Cell<i64>,
        #[property(get, set)]
        pub position: Cell<i32>,
        #[property(get, set)]
        pub suspended: Cell<bool>,
        #[property(get, set)]
        pub parent: Cell<i64>,
        #[property(get, set)]
        pub description: RefCell<String>,
        #[property(get, set)]
        pub date: Cell<i64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Task {
        const NAME: &'static str = "Task";
        type Type = super::Task;
    }

    impl ObjectImpl for Task {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }
}

glib::wrapper! {
    pub struct Task(ObjectSubclass<imp::Task>);
}

impl Task {
    pub fn new(properties: &[(&str, &dyn ToValue)]) -> Self {
        let obj = glib::Object::new::<Self>();
        obj.set_properties(properties);
        obj
    }

    pub fn duration(&self) -> i64 {
        let mut total = 0;
        for record in
            read_records(Some(self.id()), false, None, None).expect("Failed to read records")
        {
            total += record.duration();
        }
        for subtask in read_tasks(None, None, None, Some(self.id()), None, false).unwrap() {
            total += subtask.duration();
        }
        total
    }

    pub fn duration_display(&self) -> String {
        Record::duration_display(self.duration())
    }

    pub fn incomplete_record(&self) -> Option<Record> {
        let incomplete_records =
            read_records(Some(self.id()), true, None, None).expect("Failed to read records");
        match incomplete_records.len() {
            0 => None,
            1 => {
                let record = incomplete_records.get(0).unwrap().to_owned();
                record
                    .set_duration(glib::DateTime::now_local().unwrap().to_unix() - record.start());
                Some(record)
            }
            _ => panic!("The Task cannot have multiple incomplete records"),
        }
    }

    pub fn date_datetime(&self) -> Option<glib::DateTime> {
        let date = self.date();
        if date == 0 {
            None
        } else {
            Some(glib::DateTime::from_unix_local(self.date()).unwrap())
        }
    }

    pub fn date_display(datetime: &glib::DateTime) -> String {
        let now = glib::DateTime::now_local().unwrap();
        let today = glib::DateTime::new(
            &glib::TimeZone::local(),
            now.year(),
            now.month(),
            now.day_of_month(),
            0,
            0,
            0.0,
        )
        .unwrap();
        let difference = datetime.difference(&today).as_days();
        if difference == 0 {
            gettext("Today")
        } else if difference == 1 {
            gettext("Tomorrow")
        } else if today.year() == datetime.year() {
            datetime.format("%B %e, %A").unwrap().replace(" ", "")
        } else {
            datetime.format("%B %e, %Y").unwrap().replace(" ", "")
        }
    }

    pub fn different_properties<'a>(&self, other: &Self) -> Vec<&'a str> {
        let mut properties = vec![];
        if self.id() != other.id() {
            properties.push("id");
        }
        if self.name() != other.name() {
            properties.push("name");
        }
        if self.done() != other.done() {
            properties.push("done");
        }
        if self.project() != other.project() {
            properties.push("project");
        }
        if self.section() != other.section() {
            properties.push("section");
        }
        if self.position() != other.position() {
            properties.push("position");
        }
        if self.suspended() != other.suspended() {
            properties.push("suspended");
        }
        if self.parent() != other.parent() {
            properties.push("parent");
        }
        if self.description() != other.description() {
            properties.push("description");
        }
        if self.date() != other.date() {
            properties.push("date");
        }
        properties
    }

    pub fn duplicate(&self) -> Task {
        Task::new(&[
            ("id", &self.id()),
            ("name", &self.name()),
            ("done", &self.done()),
            ("project", &self.project()),
            ("section", &self.section()),
            ("position", &self.position()),
            ("suspended", &self.suspended()),
            ("parent", &self.parent()),
            ("description", &self.description()),
            ("date", &self.date()),
        ])
    }
}

impl TryFrom<&rusqlite::Row<'_>> for Task {
    type Error = rusqlite::Error;

    fn try_from(row: &rusqlite::Row) -> Result<Self, Self::Error> {
        Ok(Task::new(&[
            ("id", &row.get::<usize, i64>(0)?),
            ("name", &row.get::<usize, String>(1)?),
            ("done", &row.get::<usize, bool>(2)?),
            ("project", &row.get::<usize, i64>(3)?),
            ("section", &row.get::<usize, i64>(4)?),
            ("position", &row.get::<usize, i32>(5)?),
            ("suspended", &row.get::<usize, bool>(6)?),
            ("parent", &row.get::<usize, i64>(7)?),
            ("description", &row.get::<usize, String>(8)?),
            ("date", &row.get::<usize, i64>(9)?),
        ]))
    }
}

impl Default for Task {
    fn default() -> Self {
        Task::new(&[])
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Task {{ id: {} name: {} done: {} project: {} section: {} position: {} suspended: {} parent: {} description: {} date: {} }}",
            self.id(),
            self.name(),
            self.done(),
            self.project(),
            self.section(),
            self.position(),
            self.suspended(),
            self.parent(),
            self.description(),
            self.date()
        )
    }
}

impl ToVariant for Task {
    fn to_variant(&self) -> glib::Variant {
        glib::Variant::from((
            self.id(),
            self.name(),
            self.done(),
            self.project(),
            self.section(),
            self.position(),
            self.suspended(),
            self.parent(),
            self.description(),
            self.date(),
        ))
    }
}

impl StaticVariantType for Task {
    fn static_variant_type() -> std::borrow::Cow<'static, glib::VariantTy> {
        std::borrow::Cow::from(glib::VariantTy::new("(xsbxxibxsx)").unwrap())
    }
}

impl FromVariant for Task {
    fn from_variant(variant: &glib::Variant) -> Option<Self> {
        let (id, name, done, project, section, position, suspended, parent, description, date): (
            i64,
            String,
            bool,
            i64,
            i64,
            i32,
            bool,
            i64,
            String,
            i64,
        ) = variant.get()?;
        Some(Task::new(&[
            ("id", &id),
            ("name", &name),
            ("done", &done),
            ("project", &project),
            ("section", &section),
            ("position", &position),
            ("suspended", &suspended),
            ("parent", &parent),
            ("description", &description),
            ("date", &date),
        ]))
    }
}
