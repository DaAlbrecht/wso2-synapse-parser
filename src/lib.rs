use std::io::BufRead;

use xml::{
    name::OwnedName,
    reader::{EventReader, ParserConfig, XmlEvent},
};

pub mod ast;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct Parser<R: BufRead> {
    event_reader: EventReader<R>,
    current_event: Option<XmlEvent>,
}

impl<R: BufRead> Parser<R> {
    pub fn new(input: R) -> Self {
        let mut parser = Parser {
            event_reader: ParserConfig::new()
                .trim_whitespace(true)
                .create_reader(input),
            current_event: None,
        };

        let curr = parser.event_reader.next();

        if curr.is_ok() {
            parser.current_event = Some(curr.unwrap());
        }

        parser
    }
    pub fn parse_progarm(&mut self) -> Result<ast::Program> {
        //skip start document event
        if self.current_event
            == Some(XmlEvent::StartDocument {
                version: xml::common::XmlVersion::Version10,
                encoding: "UTF-8".to_string(),
                standalone: None,
            })
        {
            self.current_event = self.event_reader.next().ok();
        }

        let mut ast_nodes: Vec<ast::AstNode> = Vec::new();

        //parse all elements
        while self.current_event.as_ref() != Some(&XmlEvent::EndDocument) {
            let node = match self.current_event.as_ref() {
                Some(XmlEvent::StartElement { name, .. }) if name.local_name == "inSequence" => {
                    self.parse_in_sequence()
                }
                Some(XmlEvent::StartElement { .. }) => self.parse_mediator(),
                _ => {
                    return Result::Err("error".into());
                }
            };
            if node.is_ok() {
                ast_nodes.push(node.unwrap());
                println!("{:?}", ast_nodes);
            }
        }
        Result::Ok(ast::Program { ast_nodes })
    }

    //--------------------------------------------------------------------------------//

    fn parse_in_sequence(&mut self) -> Result<ast::AstNode> {
        //current event is start element of inSequence walk to the next event (start element of mediator)
        self.current_event = self.event_reader.next().ok();
        while self.current_event
            != Some(XmlEvent::EndElement {
                name: OwnedName::local("inSequence"),
            })
        {
            self.parse_mediator();
            self.current_event = self.event_reader.next().ok();
        }

        if self.current_event
            != Some(XmlEvent::EndElement {
                name: OwnedName::local("inSequence"),
            })
        {
            return Result::Err("error".into());
        }

        self.current_event = self.event_reader.next().ok();

        Result::Ok(ast::AstNode::Sequence(ast::Sequences::InSequence(
            ast::InSequence {
                mediators: Vec::new(),
            },
        )))
    }

    //--------------------------------------------------------------------------------//

    fn parse_mediator(&mut self) -> Result<ast::AstNode> {
        match self.current_event.as_ref() {
            Some(XmlEvent::StartElement { name, .. }) => match name.local_name.as_str() {
                "log" => self.parse_log_mediator(),
                "property" => self.parse_property(),
                _ => {
                    return Result::Err("error".into());
                }
            },
            _ => {
                return Result::Err("error".into());
            }
        }
    }

    fn parse_log_mediator(&mut self) -> Result<ast::AstNode> {
        let mut log_level = String::new();

        //get log level
        match self.current_event.as_ref() {
            Some(XmlEvent::StartElement { attributes, .. }) => {
                for attr in attributes {
                    if attr.name.local_name == "level" {
                        log_level = attr.value.clone();
                    }
                }
            }
            _ => {
                return Result::Err("error".into());
            }
        }

        //create log mediator node
        let mut log_mediator = ast::LogMediator {
            level: log_level,
            properties: vec![],
        };

        //current event is start element of log mediator walk to the next event (start element of property mediator)
        self.current_event = self.event_reader.next().ok();

        //parse log content properties
        while self.current_event
            != Some(XmlEvent::EndElement {
                name: OwnedName::local("log"),
            })
        {
            match self.parse_mediator() {
                Result::Ok(ast::AstNode::Mediator(ast::Mediators::Property(property))) => {
                    log_mediator.properties.push(property);
                }
                _ => {
                    return Result::Err("error".into());
                }
            }
            //skip the read property element
            self.current_event = self.event_reader.next().ok();
        }
        //error if current event is not end element of log mediator tag
        if self.current_event
            != Some(XmlEvent::EndElement {
                name: OwnedName::local("log"),
            })
        {
            return Result::Err("error".into());
        }

        self.current_event = self.event_reader.next().ok();

        Result::Ok(ast::AstNode::Mediator(ast::Mediators::Log(log_mediator)))
    }

    fn parse_property(&mut self) -> Result<ast::AstNode> {
        let mut property_name = String::new();
        let mut property_value = String::new();

        match self.current_event.as_ref() {
            Some(XmlEvent::StartElement { attributes, .. }) => {
                for attr in attributes {
                    if attr.name.local_name == "name" {
                        property_name = attr.value.clone();
                    }
                    if attr.name.local_name == "value" {
                        property_value = attr.value.clone();
                    }
                }
            }
            _ => {
                return Result::Err("error".into());
            }
        }

        //skip end element of property
        self.current_event = self.event_reader.next().ok();

        Result::Ok(ast::AstNode::Mediator(ast::Mediators::Property(
            ast::Property {
                name: property_name,
                value: property_value,
            },
        )))
    }
}

#[cfg(test)]
mod tests {
    use crate::{ast, Parser};

    #[test]
    fn test_in_sequence() {
        let input = r#"
        <inSequence>
            <log level="custom">
                <property name="/validate" value="inSequence" />
            </log>
            <class name="ch.integon.XfccMediator" />
            <log level="full" />
            <call>
              <endpoint>
                  <http method="GET" uri-template="http://httpbin:80/get">
                      <timeout>
                          <duration>15000</duration>
                          <responseAction>fault</responseAction>
                      </timeout>
                      <suspendOnFailure>
                          <errorCodes>-1</errorCodes>
                          <initialDuration>0</initialDuration>
                          <progressionFactor>1.0</progressionFactor>
                          <maximumDuration>0</maximumDuration>
                      </suspendOnFailure>
                      <markForSuspension>
                          <errorCodes>-1</errorCodes>
                      </markForSuspension>
                  </http>
              </endpoint>
            </call>
      <respond/>
      </inSequence>
        "#;

        let mut parser = Parser::new(input.as_bytes());
        parser.parse_progarm();

        assert!(false)
    }

    #[test]
    fn test_log_mediator() {
        let input = r#"
        <log level="custom">
            <property name="/validate" value="inSequence" />
        </log>
        "#;

        let mut parser = Parser::new(input.as_bytes());
        let program = parser.parse_progarm();

        println!("{:?}", program);
        assert!(program.is_ok());
        let program = program.unwrap();

        println!("{:?}", program);

        for statement in program.ast_nodes {
            match statement {
                ast::AstNode::Mediator(ast::Mediators::Log(log_mediator)) => {
                    assert_eq!(log_mediator.level, "custom");
                    assert_eq!(log_mediator.properties.len(), 1);
                    assert_eq!(log_mediator.properties[0].name, "/validate");
                    assert_eq!(log_mediator.properties[0].value, "inSequence");
                }
                _ => {
                    assert!(false);
                }
            }
        }
    }
}

/*
 * example xml
   <?xml version="1.0" encoding="uTF-8"?>
    <api context="/validate" name="validate_xfcc" xmlns="http://ws.apache.org/ns/synapse" trace="enable" statistics="enable">
      <resource methods="GET" uri-template="/">
          <inSequence>
              <log level="custom">
                  <property name="/validate" value="inSequence" />
              </log>
              <class name="ch.integon.XfccMediator" />
              <log level="full" />
              <call>
                <endpoint>
                    <http method="GET" uri-template="http://httpbin:80/get">
                        <timeout>
                            <duration>15000</duration>
                            <responseAction>fault</responseAction>
                        </timeout>
                        <suspendOnFailure>
                            <errorCodes>-1</errorCodes>
                            <initialDuration>0</initialDuration>
                            <progressionFactor>1.0</progressionFactor>
                            <maximumDuration>0</maximumDuration>
                        </suspendOnFailure>
                        <markForSuspension>
                            <errorCodes>-1</errorCodes>
                        </markForSuspension>
                    </http>
                </endpoint>
              </call>
        <respond/>
        </inSequence>
        <outSequence>
            <log level="custom">
                <property name="/health" value="outSequence" />
            </log>
            <respond />
       </outSequence>
        <faultSequence>
                <log level="custom" category="ERROR">
                    <property name="foo" value="bar" />
                </log>
                <log level="custom" category="ERROR">
                    <property name="/health" value="faultSequence" />
                    <property name="HTTP_SC" expression="$axis2:HTTP_SC" />
                    <property name="ERROR_MESSAGE" expression="$ctx:ERROR_MESSAGE" />
                    <property name="ERROR_DETAIL" expression="$ctx:ERROR_DETAIL" />
                </log>
              <respond />
        </faultSequence>
      </resource>
</api>
 *
*/

