use std::io;

use rhino2d_io::{automation::Automation, node::Node, InochiPuppet};

fn main() -> io::Result<()> {
    env_logger::Builder::new()
        .filter_module(env!("CARGO_CRATE_NAME"), log::LevelFilter::Debug)
        .filter_module("inochi_io", log::LevelFilter::Debug)
        .parse_default_env()
        .init();

    let path = match std::env::args_os().skip(1).next() {
        Some(arg) => arg,
        None => {
            eprintln!("usage: info <path-to-model>");
            std::process::exit(1);
        }
    };

    let puppet = InochiPuppet::from_path(path)?;

    println!("Metadata:");
    let meta = puppet.metadata();
    if let Some(name) = meta.name() {
        println!("- Name: {}", name);
    }
    println!("- Version: {}", meta.version());
    if let Some(v) = meta.rigger() {
        println!("- Rigger: {}", v);
    }
    if let Some(v) = meta.artist() {
        println!("- Artist: {}", v);
    }
    if let Some(v) = meta.rights() {
        println!("- Rights: {}", v);
    }
    if let Some(v) = meta.copyright() {
        println!("- Copyright: {}", v);
    }
    if let Some(v) = meta.license_url() {
        println!("- License URL: {}", v);
    }
    if let Some(v) = meta.contact() {
        println!("- Contact: {}", v);
    }
    if let Some(v) = meta.reference() {
        println!("- Reference: {}", v);
    }
    if let Some(id) = meta.thumbnail_id() {
        println!("- Thumbnail ID: {}", id);
    }
    println!("- Preserve Pixels: {}", meta.preserve_pixels());

    println!("Physics:");
    println!("- Pixels / Meter: {}", puppet.physics().pixels_per_meter());
    println!("- Gravity: {}", puppet.physics().gravity());

    println!("Nodes:");
    print_node(puppet.root_node(), 0);

    println!("{} params", puppet.params().len());
    for param in puppet.params() {
        println!("- ID: {}", param.uuid());
        println!("  Name: {}", param.name());
        println!("  Is Vec2: {}", param.is_vec2());
        println!("  Min: {:?}", param.min());
        println!("  Max: {:?}", param.max());
        println!("  Defaults: {:?}", param.defaults());
        println!("  Axis Points: {:?}", param.axis_points());
        println!("  {} Bindings", param.bindings().len());
        for binding in param.bindings() {
            println!("  - Node ID: {}", binding.node());
            println!("    Param Name: {}", binding.param_name());
            println!("    Values: {:?}", binding.values());
            println!("    Is Set: {:?}", binding.is_set());
            println!("    Interpolation: {:?}", binding.interpolate_mode());
        }
    }

    println!("{} automations", puppet.automations().len());
    for a in puppet.automations() {
        match a {
            Automation::Sine(a) => {
                println!("- Type: Sine");
                println!("  Name: {}", a.name());
                println!("  Speed: {}", a.speed());
                println!("  Function: {:?}", a.sine_type());
            }
            Automation::Physics(a) => {
                println!("- Type: Physics");
                println!("  Name: {}", a.name());
                println!("  Damping: {}", a.damping());
                println!("  Bounciness: {}", a.bounciness());
                println!("  Gravity: {}", a.gravity());
                println!("  {} Nodes", a.nodes().len());
                for node in a.nodes() {
                    println!("  - Position: {:?}", node.position());
                    println!("    Distance: {}", node.distance());
                }
            }
        }
        println!("  {} bindings", a.bindings().len());
        for binding in a.bindings() {
            println!("  - Param: {}", binding.param());
            println!("    Axis: {:?}", binding.axis());
            println!("    Range: {:?}", binding.range());
        }
    }

    println!("{} textures", puppet.textures().len());
    for (i, texture) in puppet.textures().iter().enumerate() {
        println!(
            "- #{} {:?}, {} bytes",
            i,
            texture.encoding(),
            texture.data().len(),
        );
    }

    println!("{} vendor data entries", puppet.vendor_data().len());
    for vendor in puppet.vendor_data() {
        println!("- {}: {}", vendor.name(), vendor.payload().escape_ascii());
    }

    Ok(())
}

fn print_node(node: &Node, depth: usize) {
    let indent = "  ".repeat(depth);
    let ty = match node {
        Node::Node(_) => "Node",
        Node::Drawable(_) => "Drawable",
        Node::PathDeform(_) => "PathDeform",
        Node::Part(_) => "Part",
        Node::Mask(_) => "Mask",
        Node::Composite(_) => "Composite",
        Node::SimplePhysics(_) => "SimplePhysics",
    };
    println!("{indent}- Type: {}", ty);
    println!("{indent}  ID: {}", node.uuid());
    println!("{indent}  Name: {}", node.name());
    println!("{indent}  Enabled: {}", node.enabled());
    println!("{indent}  Z-Sort: {}", node.zsort());
    println!("{indent}  Transform: {:?}", node.transform());
    println!("{indent}  Lock To Root: {}", node.lock_to_root());

    match node {
        Node::Node(_) => {}
        Node::Drawable(draw) => {
            println!("{indent}  Origin: {:?}", draw.mesh_data().origin());
            println!("{indent}  Vertices: {}", draw.mesh_data().vertex_count());
            println!("{indent}  Has UVs: {}", draw.mesh_data().uvs().is_some());
        }
        Node::PathDeform(p) => {
            println!("{indent}  Joint Origins: {:?}", p.joint_origins());
            println!("{indent}  {} bindings", p.bindings().len());
            for binding in p.bindings() {
                println!("{indent}    Target ID: {}", binding.bound_to());
                println!("{indent}    Bind Data: {:?}", binding.bind_data());
            }
        }
        Node::Part(part) => {
            println!("{indent}  Textures: {:?}", part.textures());
            println!("{indent}  Opacity: {}", part.opacity());
            println!("{indent}  Textures: {}", part.mask_threshold());
            println!("{indent}  Tint: {:?}", part.tint());
            println!("{indent}  Blend Mode: {:?}", part.blend_mode());
            if let Some(mask_mode) = part.mask_mode() {
                println!("{indent}  Mask Mode: {:?}", mask_mode);
            }
            if !part.masked_by().is_empty() {
                println!("{indent}  Masked By: {:?}", part.masked_by());
            }
        }
        Node::Mask(mask) => {
            // Same as `Drawable`
            println!("{indent}  Origin: {:?}", mask.mesh_data().origin());
            println!("{indent}  Vertices: {}", mask.mesh_data().vertex_count());
            println!("{indent}  Has UVs: {}", mask.mesh_data().uvs().is_some());
        }
        Node::Composite(comp) => {
            println!("{indent}  Blend Mode: {:?}", comp.blend_mode());
            println!("{indent}  Tint: {:?}", comp.tint());
            println!("{indent}  Mask Threshold: {:?}", comp.mask_threshold());
            println!("{indent}  Opacity: {:?}", comp.opacity());
        }
        Node::SimplePhysics(phy) => {
            println!("{indent}  Param ID: {}", phy.param());
            println!("{indent}  Model Type: {:?}", phy.model_type());
            println!("{indent}  Map Mode: {:?}", phy.map_mode());
            println!("{indent}  Gravity: {}", phy.gravity());
            println!("{indent}  Length: {}", phy.length());
            println!("{indent}  Frequency: {}", phy.frequency());
            println!("{indent}  Angle Damping: {}", phy.angle_damping());
            println!("{indent}  Length Damping: {}", phy.length_damping());
            println!("{indent}  Output Scale: {:?}", phy.output_scale());
        }
    }

    println!("{indent}  {} children", node.children().len());
    for child in node.children() {
        print_node(child, depth + 1);
    }
}
