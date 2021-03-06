use crate::entities::*;
use crate::enums::*;
use crate::helper_functions::tests::*;
use crate::objects::*;
use crate::*;

#[test]
fn follow_entity_pointer_to_object() {
    let drawing = parse_drawing(
        vec![
            "  0",
            "SECTION",
            "  2",
            "OBJECTS",
            "  0",
            "MATERIAL",
            "  5",
            "ABCD",
            "  1",
            "material-name",
            "  0",
            "ENDSEC",
            "  0",
            "SECTION",
            "  2",
            "ENTITIES",
            "  0",
            "LINE",
            "347",
            "ABCD",
            "  0",
            "ENDSEC",
            "  0",
            "EOF",
        ]
        .join("\r\n")
        .as_str(),
    );
    let entities = drawing.entities().collect::<Vec<_>>();
    let line_common = match entities[0] {
        Entity {
            ref common,
            specific: EntityType::Line(_),
        } => common,
        _ => panic!("expected a line"),
    };
    let bound_material = match line_common.get_material(&drawing).unwrap().specific {
        ObjectType::Material(ref mat) => mat,
        _ => panic!("expected a material"),
    };
    assert_eq!("material-name", bound_material.name);
}

#[test]
fn follow_object_pointer_to_entity_collection() {
    let drawing = parse_drawing(
        vec![
            "  0",
            "SECTION",
            "  2",
            "OBJECTS",
            "  0",
            "GROUP",
            "340",
            "ABCD",
            "  0",
            "ENDSEC",
            "  0",
            "SECTION",
            "  2",
            "ENTITIES",
            "  0",
            "TEXT",
            "  5",
            "ABCD",
            "  1",
            "text value",
            "  0",
            "ENDSEC",
            "  0",
            "EOF",
        ]
        .join("\r\n")
        .as_str(),
    );
    let objects = drawing.objects().collect::<Vec<_>>();
    let group = match objects[0].specific {
        ObjectType::Group(ref g) => g,
        _ => panic!("expected a group"),
    };
    let entity_collection = group.get_entities(&drawing);
    assert_eq!(1, entity_collection.len());
    let bound_text = match entity_collection[0].specific {
        EntityType::Text(ref t) => t,
        _ => panic!("expected text"),
    };
    assert_eq!("text value", bound_text.value);
}

#[test]
fn no_pointer_bound() {
    let drawing = from_section("ENTITIES", vec!["  0", "LINE"].join("\r\n").as_str());
    let entities = drawing.entities().collect::<Vec<_>>();
    match entities[0].common.get_material(&drawing) {
        None => (),
        _ => panic!("expected None"),
    }
}

#[test]
fn set_pointer_on_entity() {
    let mut drawing = Drawing::new();
    drawing.header.version = AcadVersion::R2007;
    let material = Object {
        common: Default::default(),
        specific: ObjectType::Material(Material {
            name: String::from("material-name"),
            ..Default::default()
        }),
    };
    let mut line = Entity {
        common: Default::default(),
        specific: EntityType::Line(Default::default()),
    };
    assert_eq!(0, material.common.handle);

    let material = drawing.add_object(material);
    assert_eq!(0x10, material.common.handle);
    line.common.set_material(material).ok().unwrap();
    drawing.add_entity(line);

    assert_contains(&drawing, vec!["  0", "MATERIAL", "  5", "10"].join("\r\n"));
    assert_contains(
        &drawing,
        vec![
            "  0",
            "LINE",
            "  5",
            "11",
            "100",
            "AcDbEntity",
            "  8",
            "0",
            "347",
            "10", // handle of `material`
        ]
        .join("\r\n"),
    );
}
