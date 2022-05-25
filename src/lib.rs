use std::{marker::PhantomData, collections::HashMap, any::TypeId};

use derive_builder::Builder;

pub mod args;

pub struct CliStep<PrevOut, ThisOut> {
    input: PrevOut,
    _marker: PhantomData<(PrevOut, ThisOut)>,
}

impl<PrevOut, ThisOut> CliStep<PrevOut, ThisOut> {
    pub fn new(input: PrevOut) -> Self {
        Self { input, _marker: PhantomData }
    }

    pub fn then<NextOut, F>(self, mut this_step: F) -> CliStep<ThisOut, NextOut>
    where
        F: FnMut(PrevOut) -> ThisOut
    {
        let this_out = this_step(self.input);
        CliStep::new(this_out)
    }
}

impl<PrevOut> CliStep<PrevOut, ()> {
    pub fn end(self, mut end_step: impl FnMut(PrevOut) -> ()) -> CliStep<(), ()> {
        end_step(self.input);
        CliStep::new(())
    }
}


pub struct CliDataBuilder<T> {
    data: T,
    question: String,
    default: Option<String>,
}

impl<T> CliDataBuilder<T> {
    pub fn new(data: T) -> Self {
        Self { data, question: String::new(), default: None }
    }

    pub fn ask(mut self, q: String) -> Self {
        self.question = q;
        self
    }

    pub fn ask_with_default(mut self, q: String, d: String) -> Self {
        self.question = q;
        self.default = Some(d);
        self
    }

    pub fn then(mut self, mut f: impl FnMut(&str, &mut T)) -> Self {
        let ans = Self::get_ans(&self.question, self.default.as_ref().map(|x| &**x));       
        f(&ans, &mut self.data);
        self
    }

    pub fn build(&self) -> &Self {
        self
    }

    pub fn end(self) -> T {
        self.data
    }

    fn get_ans(q: &str, d: Option<&str>) -> String {
        format!("q: {}, d: {:?}\n", q, d)
    }
}


pub trait ArgType<T> {
    fn object(settings: ArgSettings<T>) -> CliArg;
    fn extract(cli_arg: &CliArg) -> Option<&T>;
}
impl ArgType<()> for () {
    fn object(settings: ArgSettings<()>) -> CliArg {
        CliArg::Unit(None, settings)
    }

    fn extract(cli_arg: &CliArg) -> Option<&()> {
        cli_arg.unwrap_unit()
    }
}
impl ArgType<bool> for bool {
    fn object(settings: ArgSettings<bool>) -> CliArg {
        CliArg::Bool(None, settings)
    }

    fn extract(cli_arg: &CliArg) -> Option<&bool> {
        cli_arg.unwrap_bool()
    }
}
impl ArgType<i32> for i32 {
    fn object(settings: ArgSettings<i32>) -> CliArg {
        CliArg::Int(None, settings)
    }

    fn extract(cli_arg: &CliArg) -> Option<&i32> {
        cli_arg.unwrap_int()
    }
}
impl ArgType<String> for String {
    fn object(settings: ArgSettings<String>) -> CliArg {
        CliArg::String(None, settings)
    }

    fn extract(cli_arg: &CliArg) -> Option<&String> {
        cli_arg.unwrap_string()
    }
}

pub enum CliArg {
    Unit(Option<()>, ArgSettings<()>),
    Bool(Option<bool>, ArgSettings<bool>),
    Int(Option<i32>, ArgSettings<i32>),
    String(Option<String>, ArgSettings<String>),
}

impl CliArg {
    pub fn unwrap_unit(&self) -> Option<&()> {
        match self {
            CliArg::Unit(v, _) => v.as_ref(),
            _ => panic!("Not correct"),
        }
    }
    pub fn unwrap_bool(&self) -> Option<&bool> {
        match self {
            CliArg::Bool(v, _) => v.as_ref(),
            _ => panic!("Not correct"),
        }
    }
    pub fn unwrap_int(&self) -> Option<&i32> {
        match self {
            CliArg::Int(v, _) => v.as_ref(),
            _ => panic!("Not correct"),
        }
    }
    pub fn unwrap_string(&self) -> Option<&String> {
        match self {
            CliArg::String(v, _) => v.as_ref(),
            _ => panic!("Not correct"),
        }
    }
}

#[derive(Builder)]
pub struct ArgSettings<T> {
    pub optional: bool,
    pub default_value: Option<T>,
}

impl<T> Default for ArgSettings<T> {
    fn default() -> Self {
        Self {
            optional: false,
            default_value: None,
        }
    }
}

impl<T: Clone> ArgSettings<T> {
    pub fn builder() -> ArgSettingsBuilder<T> {
        ArgSettingsBuilder {
            optional: None,
            default_value: None,
        }
    }
}

pub struct CliArgsParser {
    args_ind: HashMap<String, usize>,
    args: Vec<CliArg>,
    // a: HashMap<TypeId, Vec<(CliArg, ArgSettings<()>)>>
}

impl CliArgsParser {
    pub fn new() -> Self {
        Self { args_ind: Default::default(), args: Vec::new() }
    }

    pub fn with<T>(&mut self, key: String, settings: Option<ArgSettings<T>>) -> &mut Self
    where
        T: ArgType<T>,
    {
        let settings = settings.unwrap_or_else(|| {
            ArgSettings::default()
        });
        
        let ind = self.args.len();
        self.args_ind.insert(key, ind);
        self.args.push(<T as ArgType<T>>::object(settings));

        self
    }

    pub fn parse(&mut self, cmd: &str) {
        todo!()
    }

    pub fn get<T>(&self, key: &str) -> Option<&T>
    where
        T: ArgType<T>
    {
        let ind = *self.args_ind.get(key)?;
        let cli_arg = self.args.get(ind)?;
        <T as ArgType<T>>::extract(cli_arg)
    }
}


#[cfg(test)]
mod tests {
    use crate::{CliStep, CliDataBuilder};

    #[test]
    fn it_works() {
        CliStep::new(())
            .then(|_: ()| "123".to_string())
            .then(|s: String| s.parse::<u32>().unwrap())
            .end(|n| println!("n + 10 = {}", n + 10));

        let data = CliDataBuilder::new(String::new())
            .ask("q1".to_string())
            .then(|a, data| data.push_str(a))
            .ask("q2".to_string())
            .then(|a, data| data.push_str(a))
            .end();

        println!("{}", data);
    }
}
