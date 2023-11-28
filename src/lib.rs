use std::{fmt::Display, ops::Index, str::FromStr};

#[derive(Debug, Clone)]
pub struct Table<LH, TH, T, const WIDTH: usize>
where
    LH: AsRef<str>,
    TH: AsRef<str>,
    T: AsRef<str>,
{
    header: Row<LH, TH, WIDTH>,
    alignments: Row<Alignment, Alignment, WIDTH>,
    content: Vec<Row<LH, T, WIDTH>>,
    widths: Row<usize, usize, WIDTH>,
}

impl<LH, TH, T, const WIDTH: usize> Display for Table<LH, TH, T, WIDTH>
where
    LH: AsRef<str>,
    TH: AsRef<str>,
    T: AsRef<str>,
{
    fn fmt<'x>(&'x self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let make_format_row =
            |row: &'x Row<LH, T, WIDTH>| FormatRow::new(row, &self.widths, &self.alignments);

        writeln!(
            f,
            "{}",
            FormatRow::new(&self.header, &self.widths, &self.alignments)
        )?;

        writeln!(
            f,
            "{}",
            FormatRow::new(&self.alignments, &self.widths, &self.alignments)
        )?;

        for row in &self.content {
            writeln!(f, "{}", make_format_row(row))?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

impl FromStr for Alignment {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "---" | ":---" => Ok(Self::Left),
            ":---:" => Ok(Self::Center),
            "---:" => Ok(Self::Right),
            _ => Err(()),
        }
    }
}

impl Alignment {
    fn default_row<const WIDTH: usize>() -> Row<Self, Self, WIDTH> {
        (Self::Left, [Self::Right; WIDTH]).into()
    }
}

impl AsRef<str> for Alignment {
    fn as_ref(&self) -> &str {
        match self {
            Alignment::Left => "---",
            Alignment::Center => ":---:",
            Alignment::Right => "---:",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Row<F, R, const WIDTH: usize> {
    header: F,
    content: [R; WIDTH],
}

impl<N: Default, R: Default, const WIDTH: usize> Default for Row<N, R, WIDTH> {
    fn default() -> Self {
        Self {
            header: N::default(),
            content: (0..WIDTH)
                .map(|_| R::default())
                .collect::<Vec<_>>()
                .try_into()
                .ok()
                .unwrap(),
        }
    }
}

impl<N, R, const WIDTH: usize> From<(N, [R; WIDTH])> for Row<N, R, WIDTH> {
    fn from(value: (N, [R; WIDTH])) -> Self {
        let (header, content) = value;
        Self { header, content }
    }
}

impl<N: AsRef<str>, R: AsRef<str>, const WIDTH: usize> Row<N, R, WIDTH> {
    fn widths(&self) -> Row<usize, usize, WIDTH> {
        let mut widths = Row::default();

        widths.header = self.header.as_ref().len();
        for i in 0..WIDTH {
            widths.content[i] = self.content[i].as_ref().len();
        }

        widths
    }
}

impl<T: Ord + Copy + Default, const WIDTH: usize> Row<T, T, WIDTH> {
    fn max(lhs: &Self, rhs: &Self) -> Self {
        let mut content = [T::default(); WIDTH];
        for i in 0..WIDTH {
            content[i] = T::max(lhs.content[i], rhs.content[i]);
        }
        Self {
            header: T::max(lhs.header, rhs.header),
            content,
        }
    }
}

impl<T, const WIDTH: usize> Index<usize> for Row<T, T, WIDTH> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.header,
            x => &self.content[x - 1],
        }
    }
}

struct FormatRow<'c, 'w, 'a, H, C, const WIDTH: usize> {
    content: &'c Row<H, C, WIDTH>,
    widths: &'w Row<usize, usize, WIDTH>,
    alignments: &'a Row<Alignment, Alignment, WIDTH>,
}

impl<'c, 'w, 'a, H, C, const WIDTH: usize> FormatRow<'c, 'w, 'a, H, C, WIDTH> {
    fn new(
        content: &'c Row<H, C, WIDTH>,
        widths: &'w Row<usize, usize, WIDTH>,
        alignments: &'a Row<Alignment, Alignment, WIDTH>,
    ) -> Self {
        Self {
            content,
            widths,
            alignments,
        }
    }
}

impl<'c, 'w, 'a, H: AsRef<str>, C: AsRef<str>, const WIDTH: usize> Display
    for FormatRow<'c, 'w, 'a, H, C, WIDTH>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn cell(
            f: &mut std::fmt::Formatter<'_>,
            data: impl AsRef<str>,
            width: usize,
            align: Alignment,
        ) -> std::fmt::Result {
            match align {
                Alignment::Left => write!(f, "{:<width$}", data.as_ref(), width = width),
                Alignment::Center => write!(f, "{:^width$}", data.as_ref(), width = width),
                Alignment::Right => write!(f, "{:>width$}", data.as_ref(), width = width),
            }
        }

        cell(
            f,
            &self.content.header,
            self.widths.header,
            self.alignments.header,
        )?;

        for i in 0..WIDTH {
            write!(f, " | ")?;
            cell(
                f,
                &self.content.content[i],
                self.widths.content[i],
                self.alignments.content[i],
            )?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Builder<LH, TH, T, const WIDTH: usize> {
    header: Option<Row<LH, TH, WIDTH>>,
    alignments: Option<Row<Alignment, Alignment, WIDTH>>,
    content: Vec<Row<LH, T, WIDTH>>,
    widths: Row<usize, usize, WIDTH>,
}

impl<LH, TH, T, const WIDTH: usize> Builder<LH, TH, T, WIDTH> {
    pub fn new() -> Self {
        Self {
            header: None,
            alignments: None,
            content: Vec::new(),
            widths: (0, [0; WIDTH]).into(),
        }
    }

    fn update_widths(&mut self, widths: Row<usize, usize, WIDTH>) {
        self.widths = Row::max(&self.widths, &widths);
    }
}

impl<LH: AsRef<str>, TH: AsRef<str>, T: AsRef<str>, const WIDTH: usize> Builder<LH, TH, T, WIDTH> {
    pub fn header(&mut self, header: impl Into<Row<LH, TH, WIDTH>>) {
        assert!(self.header.is_none());
        let header = header.into();

        self.update_widths(header.widths());
        self.header = Some(header);
    }

    pub fn alignments(&mut self, alignments: impl Into<Row<Alignment, Alignment, WIDTH>>) {
        assert!(self.alignments.is_none());
        let alignments = alignments.into();

        self.update_widths(alignments.widths());
        self.alignments = Some(alignments);
    }

    pub fn default_alignments(&mut self) {
        assert!(self.alignments.is_none());
        self.alignments = Some(Alignment::default_row());
    }

    pub fn row(&mut self, row: impl Into<Row<LH, T, WIDTH>>) {
        let row = row.into();

        self.update_widths(row.widths());
        self.content.push(row);
    }

    pub fn finish(self) -> Table<LH, TH, T, WIDTH> {
        Table {
            header: self.header.unwrap(),
            alignments: self.alignments.unwrap_or_else(Alignment::default_row),
            content: self.content,
            widths: self.widths,
        }
    }
}
