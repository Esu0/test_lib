use std::{
    fmt::{self, Display},
    fs::File,
    io::{self, BufWriter, Write},
    iter,
    path::{Path, PathBuf},
    str::FromStr,
};

pub struct Generator {
    gen_dir: PathBuf,
    names: Box<[(String, u32)]>,
}

impl Generator {
    pub fn new<P: ?Sized + AsRef<Path>>(gen_dir: &P) -> Self {
        Self::with_names(gen_dir, &["test", "sample"])
    }

    pub fn with_names<P: ?Sized + AsRef<Path>, T: AsRef<str>>(gen_dir: &P, names: &[T]) -> Self {
        Self {
            gen_dir: gen_dir.as_ref().to_owned(),
            names: names.iter().map(|name| (name.as_ref().to_owned(), 0)).collect(),
        }
    }

    pub fn next_writer(&mut self, idx: usize) -> BufWriter<File> {
        let (test_name, count) = &mut self.names[idx];
        let file_name = format!("{idx}_{}{:02}", test_name, count);
        *count += 1;

        self.gen_dir.push(file_name);
        let file = File::create(&self.gen_dir).expect("file creation failed");
        self.gen_dir.pop();
        BufWriter::new(file)
    }

    pub fn next_test_writer(&mut self) -> BufWriter<File> {
        assert_eq!("test", &self.names[0].0);
        self.next_writer(0)
    }

    pub fn make_with_id<I: ?Sized + Display, O: ?Sized + Display>(
        &mut self,
        input: &I,
        output: &O,
        id: usize,
    ) -> io::Result<()> {
        let (test_name, count) = &mut self.names[id];
        let file_name = format!("{id}_{test_name}{count:02}.txt");
        *count += 1;
        self.write_to(input, output, &file_name)?;

        Ok(())
    }

    pub fn make_test<I: ?Sized + Display, O: ?Sized + Display>(
        &mut self,
        input: &I,
        output: &O,
    ) -> io::Result<()> {
        assert_eq!("test", &self.names[0].0);
        self.make_with_id(input, output, 0)
    }

    pub fn make_sample<I: ?Sized + Display, O: ?Sized + Display>(
        &mut self,
        input: &I,
        output: &O,
    ) -> io::Result<()> {
        assert_eq!("sample", &self.names[1].0);
        self.make_with_id(input, output, 1)
    }

    pub fn write_to<I: ?Sized + Display, O: ?Sized + Display>(
        &mut self,
        input: &I,
        output: &O,
        name: &str,
    ) -> io::Result<()> {
        self.gen_dir.push("input");
        self.gen_dir.push(name);
        let in_file = File::create(&self.gen_dir)?;
        self.gen_dir.pop();
        self.gen_dir.pop();

        self.gen_dir.push("output");
        self.gen_dir.push(name);
        let out_file = File::create(&self.gen_dir)?;
        self.gen_dir.pop();
        self.gen_dir.pop();

        let mut writer = BufWriter::new(in_file);
        write!(writer, "{input}")?;

        let mut writer = BufWriter::new(out_file);
        write!(writer, "{output}")?;

        Ok(())
    }

    pub fn sample_num(&self) -> u32 {
        self.names[1].1
    }

    pub fn test_num(&self) -> u32 {
        self.names[0].1
    }

    pub fn num(&self, idx: usize) -> u32 {
        self.names[idx].1
    }
}

pub struct InputReader<I> {
    iterator: I,
}

impl<I: Iterator<Item = String>> InputReader<I> {
    pub fn new<IT: IntoIterator<IntoIter = I>>(iter: IT) -> Self {
        Self {
            iterator: iter.into_iter(),
        }
    }

    pub fn inner_mut(&mut self) -> &mut I {
        &mut self.iterator
    }

    pub fn inner_ref(&self) -> &I {
        &self.iterator
    }

    pub fn into_inner(self) -> I {
        self.iterator
    }

    pub fn read<In: Input>(&mut self) -> Result<In, In::Err> {
        In::read_from(self)
    }

    pub fn parse_next<T: FromStr>(&mut self) -> Option<T> {
        self.iterator.next()?.parse::<T>().ok()
    }
}

pub trait Input: Sized {
    type Err;
    fn read_from<I: Iterator<Item = String>>(
        reader: &mut InputReader<I>,
    ) -> Result<Self, Self::Err>;
}

pub struct MultiInput<I> {
    pub inputs: Box<[I]>,
}

pub struct MultiOutput<O> {
    pub outputs: Box<[O]>,
}

#[derive(Clone, Copy, Debug)]
pub enum MultiInputError<Err> {
    CaseNum,
    Child(Err),
}

impl<I: Display> Display for MultiInput<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.inputs.len())?;
        for input in &self.inputs {
            write!(f, "{input}")?;
        }
        Ok(())
    }
}

impl<O: Display> Display for MultiOutput<O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for output in &self.outputs {
            write!(f, "{output}")?;
        }
        Ok(())
    }
}

impl<T: Input> Input for MultiInput<T> {
    type Err = MultiInputError<T::Err>;
    fn read_from<I: Iterator<Item = String>>(
        reader: &mut InputReader<I>,
    ) -> Result<Self, Self::Err> {
        let t = reader
            .parse_next::<usize>()
            .ok_or_else(|| MultiInputError::CaseNum)?;
        Ok(Self {
            inputs: iter::repeat_with(|| reader.read())
                .take(t)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl<Err: Display> Display for MultiInputError<Err> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CaseNum => writeln!(f, "failed to parse case num"),
            Self::Child(err) => err.fmt(f),
        }
    }
}

impl<Err> From<Err> for MultiInputError<Err> {
    fn from(value: Err) -> Self {
        Self::Child(value)
    }
}

pub trait Solver<I> {
    type Output: Sized;
    fn solve(&self, input: &I) -> Self::Output;

    fn solve_multi(&self, inputs: &MultiInput<I>) -> MultiOutput<Self::Output> {
        MultiOutput {
            outputs: inputs
                .inputs
                .iter()
                .map(|input| self.solve(input))
                .collect(),
        }
    }
}

pub trait Verifier<I> {
    type Err;
    fn verify(&self, input: &I) -> Result<(), Self::Err>;
}
