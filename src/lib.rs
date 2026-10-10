use std::{
    collections::HashMap, error, fmt::self, fs::File, io::{self, BufWriter, Write}, iter, path::{Path, PathBuf}, str::FromStr,
};

#[derive(Clone, Debug)]
pub struct Generator<Prob, S, V> {
    problem: Prob,
    solver: S,
    verifier: V,
    gen_dir: PathBuf,
    set_id_width: usize,
    test_num_width: usize,
    id_cnt: HashMap<String, (usize, usize)>,
}

impl<Prob: Problem + Default, S: Solver<Prob> + Default, V: Verifier<Prob> + Default> Default for Generator<Prob, S, V> {
    fn default() -> Self {
        Self::new(Prob::default(), S::default(), V::default(), "testcase", 1, 2)
    }
}

#[derive(Debug)]
pub enum GenerateError<VE> {
    Io(io::Error),
    Verify(VE),
}

impl<VE: fmt::Display> fmt::Display for GenerateError<VE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(io_err) => write!(f, "{}", io_err),
            Self::Verify(verify_err) => write!(f, "verify error: {}", verify_err),
        }
    }
}

impl<VE: error::Error> error::Error for GenerateError<VE> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Io(io) => io.source(),
            Self::Verify(ve) => ve.source(),
        }
    }
}

impl<Prob: Problem, S: Solver<Prob>, V: Verifier<Prob>> Generator<Prob, S, V> {
    pub fn new<P: ?Sized + AsRef<Path>>(
        problem: Prob,
        solver: S,
        verifier: V,
        gen_dir: &P,
        set_id_width: usize,
        test_num_width: usize,
    ) -> Self {
        Self {
            id_cnt: problem.testset_list().iter().enumerate().map(|(i, &ts)| (problem.testset_name(ts).to_owned(), (i, 0))).collect(),
            problem,
            solver,
            verifier,
            gen_dir: gen_dir.as_ref().to_owned(),
            set_id_width,
            test_num_width,
        }
    }

    pub fn generate(
        &mut self,
        input: &Prob::I,
        testset: Prob::TestSet,
    ) -> Result<(), GenerateError<V::Err>> {
        self.verifier.verify(&self.problem, testset, input).map_err(GenerateError::Verify)?;
        let output = self.solver.solve(&self.problem, input);
        let ts_name = self.problem.testset_name(testset);
        let (id_mut, count_mut) = self.id_cnt.get_mut(ts_name).unwrap();
        let file_name = format!("{:03$}_{}_{:04$}.txt", *id_mut, ts_name, *count_mut, self.set_id_width, self.test_num_width);
        write_in_out(&mut self.gen_dir, input, &output, &file_name).map_err(GenerateError::Io)?;
        *count_mut += 1;
        Ok(())
    }
}

fn write_in_out<I: ?Sized + fmt::Display, O: ?Sized + fmt::Display>(
    gen_dir: &mut PathBuf,
    input: &I,
    output: &O,
    name: &str,
) -> io::Result<()> {
    gen_dir.push("input");
    gen_dir.push(name);
    let in_file = File::create(&gen_dir)?;
    gen_dir.pop();
    gen_dir.pop();

    gen_dir.push("output");
    gen_dir.push(name);
    let out_file = File::create(&gen_dir)?;
    gen_dir.pop();
    gen_dir.pop();

    let mut writer = BufWriter::new(in_file);
    write!(writer, "{input}")?;

    let mut writer = BufWriter::new(out_file);
    write!(writer, "{output}")?;

    Ok(())
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

impl<I: fmt::Display> fmt::Display for MultiInput<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.inputs.len())?;
        for input in &self.inputs {
            write!(f, "{input}")?;
        }
        Ok(())
    }
}

impl<O: fmt::Display> fmt::Display for MultiOutput<O> {
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

impl<Err: fmt::Display> fmt::Display for MultiInputError<Err> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CaseNum => writeln!(f, "failed to parse case num"),
            Self::Child(err) => err.fmt(f),
        }
    }
}

impl<Err: error::Error> error::Error for MultiInputError<Err> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::CaseNum => None,
            Self::Child(err) => err.source(),
        }
    }
}

impl<Err> From<Err> for MultiInputError<Err> {
    fn from(value: Err) -> Self {
        Self::Child(value)
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct MultiSolver<S: ?Sized> {
    inner: S,
}

impl<S> MultiSolver<S> {
    pub fn new(solver: S) -> Self {
        Self { inner: solver }
    }
}

impl<S: ?Sized> MultiSolver<S> {
    pub fn from_ref(solver: &S) -> &Self {
        unsafe { &*(solver as *const _ as *const Self) }
    }

    pub fn from_mut(solver: &mut S) -> &mut Self {
        unsafe { &mut *(solver as *mut _ as *mut Self) }
    }
}

pub trait Problem {
    type I: Input + fmt::Display;
    type O: fmt::Display;
    type TestSet: Copy;

    fn testset_list(&self) -> &[Self::TestSet];
    fn testset_name(&self, testset: Self::TestSet) -> &str;
}

#[derive(Clone, Copy, Debug)]
pub struct MultiProblem<P> {
    inner: P,
}

impl<P: Problem> Problem for MultiProblem<P> {
    type I = MultiInput<P::I>;
    type O = MultiOutput<P::O>;
    type TestSet = P::TestSet;

    fn testset_list(&self) -> &[Self::TestSet] {
        self.inner.testset_list()
    }
    fn testset_name(&self, testset: Self::TestSet) -> &str {
        self.inner.testset_name(testset)
    }
}

pub trait Solver<P: Problem> {
    fn solve(&self, problem: &P, input: &P::I) -> P::O;
}

impl<P: Problem, S: Solver<P>> Solver<MultiProblem<P>> for MultiSolver<S> {
    fn solve(&self, problem: &MultiProblem<P>, input: &<MultiProblem<P> as Problem>::I) -> <MultiProblem<P> as Problem>::O {
        MultiOutput {
            outputs: input.inputs.iter().map(|i| self.inner.solve(&problem.inner, i)).collect()
        }
    }
}

pub trait Verifier<P: Problem> {
    type Err;
    fn verify(&self, problem: &P, test_set: P::TestSet, input: &P::I) -> Result<(), Self::Err>;
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
}