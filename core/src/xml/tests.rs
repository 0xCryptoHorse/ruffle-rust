//! XML tests

use crate::xml;
use crate::xml::{XMLDocument, XMLName};
use gc_arena::rootless_arena;

/// Tests very basic parsing of a single-element document.
#[test]
fn parse_single_element() {
    rootless_arena(|mc| {
        let xml = XMLDocument::from_str(mc, "<test></test>").expect("Parsed document");
        dbg!(xml);
        let mut roots = xml
            .as_node()
            .children()
            .expect("Parsed document should be capable of having child nodes");

        let root = roots.next().expect("Parsed document should have a root");
        assert_eq!(root.node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.tag_name(), Some(XMLName::from_str("test").unwrap()));

        let mut root_children = root.children().unwrap();
        assert!(root_children.next().is_none());

        assert!(roots.next().is_none());
    })
}

/// Tests double-ended traversal of child nodes via DoubleEndedIterator.
#[test]
fn double_ended_children() {
    rootless_arena(|mc| {
        let xml = XMLDocument::from_str(
            mc,
            "<test></test><test2></test2><test3></test3><test4></test4><test5></test5>",
        )
        .expect("Parsed document");

        let mut roots = xml
            .as_node()
            .children()
            .expect("Parsed document should be capable of having child nodes");

        let root = roots.next().expect("Should have first root");
        assert_eq!(root.node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.tag_name(), Some(XMLName::from_str("test").unwrap()));

        let root = roots.next_back().expect("Should have last root");
        assert_eq!(root.node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.tag_name(), Some(XMLName::from_str("test5").unwrap()));

        let root = roots.next().expect("Should have next root");
        assert_eq!(root.node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.tag_name(), Some(XMLName::from_str("test2").unwrap()));

        let root = roots.next_back().expect("Should have second-to-last root");
        assert_eq!(root.node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.tag_name(), Some(XMLName::from_str("test4").unwrap()));

        let root = roots.next().expect("Should have next root");
        assert_eq!(root.node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.tag_name(), Some(XMLName::from_str("test3").unwrap()));

        assert!(roots.next().is_none());
        assert!(roots.next_back().is_none());
    })
}
