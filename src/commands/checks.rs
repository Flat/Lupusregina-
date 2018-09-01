use serenity::framework::standard::{Args, CommandOptions};
use serenity::model::channel::Message;
use serenity::prelude::*;

use util;

fn owner_check(ctx: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> bool {
    if let Ok(owner) = util::get_owner(ctx) {
        msg.author.id == owner
    } else {
        false
    }
}

fn admin_check(_: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> bool {
    if let Some(member) = msg.member() {
        if let Ok(permissions) = member.permissions() {
            return permissions.administrator();
        }
    }

    false
}
