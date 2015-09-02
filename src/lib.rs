extern crate rustc_serialize;
extern crate rand;

use std::collections::HashMap;
use rand::Rng;
use std::fmt::Write;
use std::io::Read;
use std::fmt::Display;
use rustc_serialize::json;

#[derive(Debug)]
pub struct Error(String);

impl<T> From<T> for Error where T: Display {
    fn from(other: T) -> Error {
        Error(format!("{}", other))
    }
}

#[derive(RustcDecodable,PartialEq)]
pub struct Chain {
    nodes: HashMap<String,Node>,
    start: String,
    end: String,
}

#[derive(RustcDecodable,PartialEq)]
pub struct Node {
    text: String,
    links: HashMap<String,f32>,
}

#[derive(RustcDecodable,PartialEq)]
pub struct Link {
    next: String,
    weight: f32,
}

impl Chain {
    pub fn from_file(path: &str) -> Result<Chain,Error> {
        let mut buf = String::new();
        let mut file = try!(std::fs::File::open(path));
        try!(file.read_to_string(&mut buf));

        Chain::from_str(&buf)
    }

    pub fn verify(&self) -> Option<String> {
        match self.nodes.get(&self.start) {
            Some(_) => {},
            None => return Some("start node not found".into())
        }

        match self.nodes.get(&self.end) {
            Some(_) => return Some("non-dummy end node".into()),
            None => {},
        }

        for (node_name, node) in self.nodes.iter() {
            let mut sum: f32 = 0.0;
            for (link, weight) in node.links.iter() {
                sum += *weight;
                if self.end != *link {
                    match self.nodes.get(link) {
                        Some(_) => {},
                        None => return Some(format!("link {} in node {} has no definition", link, node_name))
                    }
                }
            }
            if sum != 1.0 {
                return Some(format!("links in node {} do not sum to 1", node_name))
            }
        }

        None
    }

    pub fn from_str(buf: &str) -> Result<Chain,Error> {
        let chain: Chain = try!(json::decode(buf));
        match chain.verify() {
            None => Ok(chain),
            Some(error) => Err(error.into()),
        }
    }
}

pub struct ChainIter {
    chain: Chain,
    rng: Box<Rng>,
}

impl Iterator for ChainIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut node = self.chain.nodes.get(&self.chain.start).unwrap();
        let mut buf = node.text.clone();
        'outer: loop {
            let r: f32 = self.rng.gen_range(0.0, 1.0);
            let mut sum: f32 = 0.0;
            'inner: for (k, v) in node.links.iter() {
                sum += *v;
                if r <= sum {
                    if *k == self.chain.end {
                        break 'outer;
                    }
                    node = self.chain.nodes.get(k).unwrap();
                    buf.write_str(&node.text.clone());
                    break 'inner;
                }
            }
        }
        Some(buf)
    }
}

impl IntoIterator for Chain {
    type Item = String;
    type IntoIter = ChainIter;

    fn into_iter(self) -> Self::IntoIter {
        ChainIter{ chain: self, rng: Box::new(rand::thread_rng()) }
    }
}

#[cfg(test)]
mod tests {
    use rustc_serialize::json;
    use std::collections::HashMap;

    #[test]
    fn test_decode() {
        let in_str = r##"
{
  "nodes": {
    "start": {
      "text": "a",
      "links": {
        "next": 0.50,
        "stop": 0.50
      }
    },
    "next": {
      "text": "b",
      "links": {
        "start": 1.00
      }
    }
  },
  "start": "start",
  "end":"stop"
}
"##;
        let mut test_chain = ::Chain{
            nodes: HashMap::new(),
            start: "start".into(),
            end: "stop".into(),
        };
        test_chain.nodes.insert("start".into(),::Node{
            text: "a".into(),
            links: {
                let mut ls = HashMap::new();
                ls.insert("stop".into(), 0.50);
                ls.insert("next".into(), 0.50);
                ls
            }
        });
        test_chain.nodes.insert("next".into(), ::Node{
            text: "b".into(),
            links: {
                let mut ls = HashMap::new();
                ls.insert("start".into(), 1.00);
                ls
            }
        });
        let chain: ::Chain = json::decode(&in_str).unwrap();
        assert!(chain == test_chain);
    }
}


#[test]
fn it_works() {
    assert!(0.50 + 0.22 + 0.28 == 1.0);
}
