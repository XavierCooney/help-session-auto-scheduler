const SEPARATOR: char = '\t';

pub struct Tsv {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

pub struct TsvIterator<'a> {
    tsv: &'a Tsv,
    row_iterator: std::ops::Range<usize>,
}

pub struct Row<'a> {
    tsv: &'a Tsv,
    row_index: usize,
}

impl<'a> Iterator for TsvIterator<'a> {
    type Item = Row<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let row_index = self.row_iterator.next()?;
        Some(Row {
            tsv: self.tsv,
            row_index,
        })
    }
}

impl<'a> IntoIterator for &'a Tsv {
    type Item = Row<'a>;
    type IntoIter = TsvIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TsvIterator {
            tsv: self,
            row_iterator: 0..self.num_rows(),
        }
    }
}

impl<'a> Row<'a> {
    pub fn get(&self, column: &str) -> &'a str {
        // this isn't fast but that doesn't matter
        let index = self
            .tsv
            .headers
            .iter()
            .position(|header| header == column)
            .unwrap_or_else(|| panic!("no column {column:?} in this TSV "));
        &self.tsv.rows[self.row_index][index]
    }

    pub fn all_fields(&self) -> &[String] {
        &self.tsv.rows[self.row_index]
    }
}

impl Tsv {
    pub fn num_rows(&self) -> usize {
        self.rows.len()
    }

    pub fn from_string(contents: &str) -> Self {
        let mut lines = contents.lines();
        let headers = lines.next().unwrap();

        let split_headers: Vec<String> = headers.split(SEPARATOR).map(String::from).collect();
        let num_fields = split_headers.len();

        let rows = lines.map(|line| {
            let split_line: Vec<String> = line.split(SEPARATOR).map(String::from).collect();
            if split_line.len() != num_fields {
                panic!(
                    "TSV line {line:?} has {} fields, but expected {num_fields} fields",
                    split_line.len()
                );
            }
            split_line
        });

        Tsv {
            headers: split_headers,
            rows: rows.collect(),
        }
    }
}
