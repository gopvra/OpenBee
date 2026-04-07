//! Rust-to-Lua bindings — register engine types as Lua userdata and provide
//! global helper functions.

use mlua::prelude::*;

// ---------------------------------------------------------------------------
// Lua-visible wrapper types
// ---------------------------------------------------------------------------

/// 2-D vector exposed to Lua.
#[derive(Debug, Clone, Copy)]
pub struct LuaVec2 {
    pub x: f32,
    pub y: f32,
}

impl LuaUserData for LuaVec2 {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_, this| Ok(this.x));
        fields.add_field_method_set("x", |_, this, val: f32| {
            this.x = val;
            Ok(())
        });
        fields.add_field_method_get("y", |_, this| Ok(this.y));
        fields.add_field_method_set("y", |_, this, val: f32| {
            this.y = val;
            Ok(())
        });
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("length", |_, this, ()| {
            Ok((this.x * this.x + this.y * this.y).sqrt())
        });

        methods.add_method("normalized", |_, this, ()| {
            let len = (this.x * this.x + this.y * this.y).sqrt();
            if len == 0.0 {
                Ok(LuaVec2 { x: 0.0, y: 0.0 })
            } else {
                Ok(LuaVec2 {
                    x: this.x / len,
                    y: this.y / len,
                })
            }
        });

        methods.add_method("dot", |_, this, other: LuaVec2| {
            Ok(this.x * other.x + this.y * other.y)
        });

        methods.add_method("distance_to", |_, this, other: LuaVec2| {
            let dx = this.x - other.x;
            let dy = this.y - other.y;
            Ok((dx * dx + dy * dy).sqrt())
        });

        methods.add_meta_method(LuaMetaMethod::Add, |_, this, other: LuaVec2| {
            Ok(LuaVec2 {
                x: this.x + other.x,
                y: this.y + other.y,
            })
        });

        methods.add_meta_method(LuaMetaMethod::Sub, |_, this, other: LuaVec2| {
            Ok(LuaVec2 {
                x: this.x - other.x,
                y: this.y - other.y,
            })
        });

        methods.add_meta_method(LuaMetaMethod::Mul, |_, this, scalar: f32| {
            Ok(LuaVec2 {
                x: this.x * scalar,
                y: this.y * scalar,
            })
        });

        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| {
            Ok(format!("Vec2({}, {})", this.x, this.y))
        });
    }
}

impl FromLua for LuaVec2 {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::UserData(ud) => ud.borrow::<LuaVec2>().map(|v| *v),
            _ => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "LuaVec2".to_string(),
                message: Some("expected Vec2 userdata".to_string()),
            }),
        }
    }
}

/// Lightweight entity handle exposed to Lua.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LuaEntity {
    pub id: u32,
    pub generation: u32,
}

impl LuaUserData for LuaEntity {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("id", |_, this| Ok(this.id));
        fields.add_field_method_get("generation", |_, this| Ok(this.generation));
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, |_, this, other: LuaEntity| {
            Ok(this.id == other.id && this.generation == other.generation)
        });

        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| {
            Ok(format!("Entity({}v{})", this.id, this.generation))
        });
    }
}

impl FromLua for LuaEntity {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::UserData(ud) => ud.borrow::<LuaEntity>().map(|e| *e),
            _ => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "LuaEntity".to_string(),
                message: Some("expected Entity userdata".to_string()),
            }),
        }
    }
}

/// RGBA colour exposed to Lua.
#[derive(Debug, Clone, Copy)]
pub struct LuaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl LuaUserData for LuaColor {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("r", |_, this| Ok(this.r));
        fields.add_field_method_set("r", |_, this, val: u8| {
            this.r = val;
            Ok(())
        });
        fields.add_field_method_get("g", |_, this| Ok(this.g));
        fields.add_field_method_set("g", |_, this, val: u8| {
            this.g = val;
            Ok(())
        });
        fields.add_field_method_get("b", |_, this| Ok(this.b));
        fields.add_field_method_set("b", |_, this, val: u8| {
            this.b = val;
            Ok(())
        });
        fields.add_field_method_get("a", |_, this| Ok(this.a));
        fields.add_field_method_set("a", |_, this, val: u8| {
            this.a = val;
            Ok(())
        });
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| {
            Ok(format!("Color({}, {}, {}, {})", this.r, this.g, this.b, this.a))
        });
    }
}

impl FromLua for LuaColor {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::UserData(ud) => ud.borrow::<LuaColor>().map(|c| *c),
            _ => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "LuaColor".to_string(),
                message: Some("expected Color userdata".to_string()),
            }),
        }
    }
}

/// Axis-aligned rectangle exposed to Lua.
#[derive(Debug, Clone, Copy)]
pub struct LuaRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl LuaUserData for LuaRect {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_, this| Ok(this.x));
        fields.add_field_method_set("x", |_, this, val: f32| {
            this.x = val;
            Ok(())
        });
        fields.add_field_method_get("y", |_, this| Ok(this.y));
        fields.add_field_method_set("y", |_, this, val: f32| {
            this.y = val;
            Ok(())
        });
        fields.add_field_method_get("w", |_, this| Ok(this.w));
        fields.add_field_method_set("w", |_, this, val: f32| {
            this.w = val;
            Ok(())
        });
        fields.add_field_method_get("h", |_, this| Ok(this.h));
        fields.add_field_method_set("h", |_, this, val: f32| {
            this.h = val;
            Ok(())
        });
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("contains", |_, this, (px, py): (f32, f32)| {
            Ok(px >= this.x && px <= this.x + this.w && py >= this.y && py <= this.y + this.h)
        });

        methods.add_method("intersects", |_, this, other: LuaRect| {
            Ok(this.x < other.x + other.w
                && this.x + this.w > other.x
                && this.y < other.y + other.h
                && this.y + this.h > other.y)
        });

        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| {
            Ok(format!("Rect({}, {}, {}, {})", this.x, this.y, this.w, this.h))
        });
    }
}

impl FromLua for LuaRect {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::UserData(ud) => ud.borrow::<LuaRect>().map(|r| *r),
            _ => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "LuaRect".to_string(),
                message: Some("expected Rect userdata".to_string()),
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

/// Register all built-in Lua constructors and global helper functions.
pub fn register_all(lua: &Lua) -> mlua::Result<()> {
    let globals = lua.globals();

    // Constructor: Vec2(x, y)
    globals.set(
        "Vec2",
        lua.create_function(|_, (x, y): (f32, f32)| Ok(LuaVec2 { x, y }))?,
    )?;

    // Constructor: Entity(id, generation)
    globals.set(
        "Entity",
        lua.create_function(|_, (id, generation): (u32, u32)| {
            Ok(LuaEntity { id, generation })
        })?,
    )?;

    // Constructor: Color(r, g, b, a?)
    globals.set(
        "Color",
        lua.create_function(|_, (r, g, b, a): (u8, u8, u8, Option<u8>)| {
            Ok(LuaColor {
                r,
                g,
                b,
                a: a.unwrap_or(255),
            })
        })?,
    )?;

    // Constructor: Rect(x, y, w, h)
    globals.set(
        "Rect",
        lua.create_function(|_, (x, y, w, h): (f32, f32, f32, f32)| {
            Ok(LuaRect { x, y, w, h })
        })?,
    )?;

    // Utility: print override that goes through tracing
    globals.set(
        "log",
        lua.create_function(|_, msg: String| {
            tracing::info!(target: "lua", "{msg}");
            Ok(())
        })?,
    )?;

    // Utility: clamp(value, min, max)
    globals.set(
        "clamp",
        lua.create_function(|_, (v, lo, hi): (f64, f64, f64)| Ok(v.clamp(lo, hi)))?,
    )?;

    // Utility: lerp(a, b, t)
    globals.set(
        "lerp",
        lua.create_function(|_, (a, b, t): (f64, f64, f64)| Ok(a + (b - a) * t))?,
    )?;

    Ok(())
}
