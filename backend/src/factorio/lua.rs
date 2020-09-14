use crate::types::FactorioPlayer;
use rlua::Lua;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

pub fn execute_lua_plan(plan_name: &str) -> anyhow::Result<()> {
    let lua_path_str = format!("../plans/{}.lua", plan_name);
    let lua_path = Path::new(&lua_path_str);

    if !lua_path.exists() {
        let path = env::current_dir()?;
        println!("The current directory is {}", path.display());
        anyhow::bail!("plan {} not found at {}", plan_name, lua_path_str);
    }
    let lua_code = read_to_string(lua_path)?;
    let lua = Lua::new();

    let players: Vec<FactorioPlayer> = vec![
        FactorioPlayer {
            player_id: 1,
            ..Default::default()
        },
        FactorioPlayer {
            player_id: 2,
            ..Default::default()
        },
        FactorioPlayer {
            player_id: 3,
            ..Default::default()
        },
        FactorioPlayer {
            player_id: 4,
            ..Default::default()
        },
    ];
    // let players: Vec<String> = vec!["a".into()];

    lua.context(|lua_ctx| {
        let globals = lua_ctx.globals();
        globals
            .set("all_bots", rlua_serde::to_value(lua_ctx, &players).unwrap())
            .unwrap();
    });
    lua.context(|lua_context| lua_context.load(&lua_code).exec())?;

    // lua.context(|lua_ctx: Context| {
    //     // You can get and set global variables.  Notice that the globals table here is a permanent
    //     // reference to _G, and it is mutated behind the scenes as Lua code is loaded.  This API is
    //     // based heavily around sharing and internal mutation (just like Lua itself).
    //
    //     let globals = lua_ctx.globals();
    //
    //     globals.set("string_var", "hello")?;
    //     globals.set("int_var", 42)?;
    //
    //     let check_equal =
    //         lua_ctx.create_function(|_, (list1, list2): (Vec<String>, Vec<String>)| {
    //             // This function just checks whether two string lists are equal, and in an inefficient way.
    //             // Lua callbacks return `rlua::Result`, an Ok value is a normal return, and an Err return
    //             // turns into a Lua 'error'.  Again, any type that is convertible to Lua may be returned.
    //             Ok(list1 == list2)
    //         })?;
    //     globals.set("check_equal", check_equal)?;
    //
    //     lua_ctx
    //         .load(
    //             r#"
    //             global = 'foo'..'bar'
    //         "#,
    //         )
    //         .set_name("example code")?
    //         .exec()?;
    //
    //     Ok(())
    // })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_plan() {
        execute_lua_plan("arturh").unwrap();
    }
}
