use ratatui::widgets::Row;

pub fn create_row_from_results<'a>(results: Vec<Vec<String>>) -> Vec<Row<'a>> {
    results
        .into_iter()
        .map(|result| Row::new(result))
        .collect::<Vec<Row>>()
}
