use super::schema::links;

#[derive(Queryable)]
pub struct Link {
    pub id: i32,
    pub original_link: String,
    pub short_code: String,
    pub created: chrono::NaiveDateTime,
}

impl Link {
    pub fn created_formatted(&self) -> String {
        self.created.format("%Y/%m/%d").to_string()
    }
}

#[derive(Insertable)]
#[table_name = "links"]
pub struct NewLink<'a> {
    pub original_link: &'a str,
    pub short_code: &'a str,
}
