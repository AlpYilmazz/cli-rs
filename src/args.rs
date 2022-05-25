use std::{env, fs::File, fmt::Debug};
use std::collections::HashMap;
use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug)]
pub struct ArgSettings<T: Debug> {
    optional: bool,
    default_val: Option<T>,
}

impl<T: Debug> Default for ArgSettings<T> {
    fn default() -> Self {
        Self {
            optional: false,
            default_val: None,
        }
    }
}

impl<T: Clone + Debug> ArgSettings<T> {
    pub fn apply(&self, vals: &mut Vec<T>) -> Result<(), ()> {

        let mut ok = true;
        if vals.is_empty() { // no val was given
            if self.optional { // arg was optional
                match &self.default_val {
                    Some(d) => { // default val was provided
                        vals.push(d.clone());
                    },
                    None => {},
                }
            }
            else { // arg was not optional
                match &self.default_val {
                    Some(d) => { // default val was provided
                        vals.push(d.clone());
                    },
                    None => ok = false, // default val was not provided
                }
            }
        }
        
        if ok { Ok(()) } else { Err(()) }
    }
}

#[derive(Debug)]
pub enum Arg {
    Bool { vals: Vec<bool>, settings: ArgSettings<bool> },
    Int { vals: Vec<i32>, settings: ArgSettings<i32> },
    String { vals: Vec<String>, settings: ArgSettings<String> },
}

impl Arg {
    pub fn apply_settings(&mut self) -> Result<(), ()> {
        match self {
            Arg::Bool { vals, settings } => settings.apply(vals)?,
            Arg::Int { vals, settings } => settings.apply(vals)?,
            Arg::String { vals, settings } => settings.apply(vals)?,
        };
        Ok(())
    }
}

#[derive(Debug)]
pub enum ArgError {
    WrongKey,
    WrongType,
}

#[derive(Default, Debug)]
pub struct CliArgs {
    keys: HashMap<String, usize>,
    args: Vec<Arg>,
}

impl CliArgs {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with(&mut self, schema: &str) -> &mut Self {
        let (key_l, key_s, arg_base) = Self::parse_schema(schema);        
        let ind = self.args.len();
    
        if let Some(key_s) = key_s {
            self.keys.insert(key_s, ind);   
        }
        if let Some(key_l) = key_l {
            self.keys.insert(key_l, ind);   
        }
        self.args.push(arg_base);
    
        self
    }

    pub fn help(&self) -> String {
        todo!()
    }

    pub fn parse_cmd(&mut self) -> Result<(), ()> {
        let args_vec: Vec<String> = env::args().collect();

        if args_vec.is_empty() {
            return Ok(());
        }

        let f = File::open(&args_vec[0]);
        let mut start = 0;
        if let Ok(_) = f {
            start = 1; // first arg is the program path, skip it
        }

        let mut prev_key = String::new();
        for arg_str in args_vec.iter().skip(start) {
            if Self::is_long_key(arg_str) {
                let (key_l, val) = arg_str.split_once("=").unwrap_or_else(|| (&arg_str, ""));
                let arg = self.get_mut_arg(&key_l).expect("key not found");
                match arg {
                    Arg::Bool { vals, .. } => {
                        assert!(val.is_empty());
                        vals.push(true);
                    },
                    Arg::Int { vals, .. } => vals.push(val.parse().map_err(|e| ())?),
                    Arg::String { vals, .. } => vals.push(val.to_string()),
                }
            }
            else if Self::is_short_key(arg_str) {
                let arg = self.get_mut_arg(&arg_str).expect("key not found");
                if let Arg::Bool { vals, .. } = arg {
                    vals.push(true);
                }
                else {
                    prev_key.push_str(arg_str);
                }
            }
            else { // is val
                let arg = self.get_mut_arg(&prev_key).expect("key not found");
                match arg {
                    Arg::Int { vals, .. } => vals.push(arg_str.parse().map_err(|e| ())?),
                    Arg::String { vals, .. } => vals.push(arg_str.to_string()),
                    _ => panic!("How did I end up here?"),
                }
                prev_key.clear();
            }
        }

        dbg!(&self.keys);

        for arg in self.args.iter_mut() {
            arg.apply_settings()?;
        }

        Ok(())
    }

    const KV_REGEX: &'static str = r#"(((?P<key_l>\s+--\w+)=)|(?P<key_s>\s+-\w+\s+))(?P<val>(\S+)|("[^"]*"))?"#;

    // TODO
    pub fn parse(&mut self, args_line: &str) -> Result<(), ()> {
        todo!("Probably not todo");
        lazy_static! {
            static ref RE: Regex = Regex::new(CliArgs::KV_REGEX).unwrap();
        }
        let captures = RE.captures_iter(&args_line);

        for cap in captures {
            let key = cap.name("key_l").unwrap_or_else(|| cap.name("key_s").unwrap());
            let val = cap.name("val");

            let arg = self.get_mut_arg(key.as_str()).map(|a| Ok(a)).unwrap_or(Err(()))?;
            match arg {
                Arg::Bool { vals, .. } => vals.push(true),
                Arg::Int { vals, .. } => vals.push(val.unwrap().as_str().parse().map_err(|_| ())?),
                Arg::String { vals, .. } => vals.push(val.unwrap().as_str().to_string()),
            }
        }

        for arg in self.args.iter_mut() {
            arg.apply_settings()?;
        }

        Ok(())
    }

    pub fn get_bool(&self, key: &str) -> Result<Option<bool>, ArgError> {
        self.get_bool_multi(key).map(|vs| vs.get(0).cloned())
    }

    pub fn get_int(&self, key: &str) -> Result<Option<i32>, ArgError> {
        self.get_int_multi(key).map(|vs| vs.get(0).cloned())
    }

    pub fn get_string(&self, key: &str) -> Result<Option<String>, ArgError> {
        self.get_string_multi(key).map(|vs| vs.get(0).cloned())
    }
    
    pub fn get_str(&self, key: &str) -> Result<Option<&str>, ArgError> {
        self.get_string_multi(key).map(|vs| vs.get(0).map(|s| &**s))
    }
    
    pub fn unwrap_bool(&self, key: &str) -> bool {
        self.get_bool(key).unwrap().unwrap()
    }

    pub fn unwrap_int(&self, key: &str) -> i32 {
        self.get_int(key).unwrap().unwrap()
    }

    pub fn unwrap_string(&self, key: &str) -> String {
        self.get_string(key).unwrap().unwrap()
    }

    pub fn unwrap_str(&self, key: &str) -> &str {
        self.get_str(key).unwrap().unwrap()
    }

    pub fn get_bool_multi(&self, key: &str) -> Result<&[bool], ArgError> {
        let arg = self.get_arg(key).ok_or(ArgError::WrongKey)?;
        match arg {
            Arg::Bool { vals, .. } => Ok(vals),
            _ => Err(ArgError::WrongType),
        }
    }

    pub fn get_int_multi(&self, key: &str) -> Result<&[i32], ArgError> {
        let arg = self.get_arg(key).ok_or(ArgError::WrongKey)?;
        match arg {
            Arg::Int { vals, .. } => Ok(vals),
            _ => Err(ArgError::WrongType),
        }
    }

    pub fn get_string_multi(&self, key: &str) -> Result<&[String], ArgError> {
        let arg = self.get_arg(key).ok_or(ArgError::WrongKey)?;
        match arg {
            Arg::String { vals, .. } => Ok(vals),
            _ => Err(ArgError::WrongType),
        }
    }

    pub fn unwrap_bool_multi(&self, key: &str) -> &[bool] {
        self.get_bool_multi(key).unwrap()//.iter().map(|e| e.clone()).collect()
    }

    pub fn unwrap_int_multi(&self, key: &str) -> &[i32] {
        self.get_int_multi(key).unwrap()//.iter().map(|e| e.clone()).collect()
    }

    pub fn unwrap_string_multi(&self, key: &str) -> &[String] {
        self.get_string_multi(key).unwrap()//.iter().map(|e| e.clone()).collect()
    }


    fn is_long_key(s: &str) -> bool {
        s.starts_with("--")
    }

    fn is_short_key(s: &str) -> bool {
        s.starts_with("-") && (!s.starts_with("--"))
    }

    fn get_arg(&self, key: &str) -> Option<&Arg> {
        self.args.get(*self.keys.get(key)?)
    }

    fn get_mut_arg(&mut self, key: &str) -> Option<&mut Arg> {
        self.args.get_mut(*self.keys.get(key)?)
    }

    // const SCHEMA_REGEX: &'static str = r#"((?P<kl>--[\w_-]+)|(?P<ks>-[\w_-]+)|(?P<kls>--[\w_-]+/-[\w_-]+))=(?P<type>[bis])\??(:(?P<default_val>.+))?"#;
    const SCHEMA_REGEX: &'static str = r#"((?P<kl>--[\w_-]+)|(?P<ks>-[\w_-]+)|(?P<kls>--[\w_-]+/-[\w_-]+))=(?P<type>[bis])\??"#;

    fn parse_schema(schema: &str) -> (Option<String>, Option<String>, Arg) {
        let split = schema.split_once("::>");
        let mut default_val: Option<String> = None;
        if let Some((_, default_val_0)) = split {
            default_val = Some(default_val_0.to_string());
        }
        let schema: String = schema.split_whitespace().collect();

        lazy_static! {
            static ref RE: Regex = Regex::new(CliArgs::SCHEMA_REGEX).unwrap();
        }
        let captures = RE.captures(&schema).unwrap();
        let kls = captures.name("kls");
        let kl = captures.name("kl");
        let ks = captures.name("ks");
        let arg_type = captures.name("type").unwrap();
        let optional = captures.name("optional");
        //let default_val = captures.name("default_val");

        let to_string_op_t = |(s1, s2): (&str, &str)| {
            (Some(s1.to_string()), Some(s2.to_string()))
        };

        let (key_l, key_s) = match kls {
            Some(kls) => to_string_op_t(kls.as_str().split_once("/").unwrap()),
            None => (kl.map(|s| s.as_str().to_string()),
                    ks.map(|s| s.as_str().to_string())),
        };

        let optional = optional.map_or(false, |_| true);
        let mut arg = match arg_type.as_str() {
            "b" => {
                Arg::Bool {
                    vals: Vec::new(),
                    settings: ArgSettings {
                        optional,
                        default_val: default_val.map(|d| d.as_str().parse().unwrap())
                    },
                }
            },
            "i" => {
                Arg::Int {
                    vals: Vec::new(),
                    settings: ArgSettings {
                        optional,
                        default_val: default_val.map(|d| d.as_str().parse().unwrap())
                    },
                }
            },
            "s" => {
                Arg::String {
                    vals: Vec::new(),
                    settings: ArgSettings {
                        optional,
                        default_val: default_val.map(|d| d.as_str().parse().unwrap())
                    },
                }
            },
            _ => panic!("Parse error"),
        };

        (key_l, key_s, arg)
    }
}

#[cfg(test)]
mod tests {
    use super::{CliArgs, ArgError};


    #[test]
    fn cli_args_use() {
        let cmd_line = "";
        let mut args = CliArgs::new();
        args
            .with("--name/-n=s")
            .with("--age/-a = i? ::>18")    
            .with("--adult=b?")    
            .parse(cmd_line)
            .unwrap();

        let name = args.get_str("--name");
        let age = args.get_int("-a");
        let is_adult = args.get_bool("--adult");
        dbg!(name);
        dbg!(age);
        dbg!(is_adult);
    }

}