use super::{team::Team, ApplicationFlags, InstallParams};
use crate::{
    id::{
        marker::{ApplicationMarker, GuildMarker, OauthSkuMarker},
        Id,
    },
    user::User,
    util::image_hash::ImageHash,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Application {
    pub bot_public: bool,
    pub bot_require_code_grant: bool,
    pub cover_image: Option<ImageHash>,
    /// Application's default custom authorization link, if enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_install_url: Option<String>,
    pub description: String,
    pub guild_id: Option<Id<GuildMarker>>,
    /// Public flags of the application.
    pub flags: Option<ApplicationFlags>,
    pub icon: Option<ImageHash>,
    pub id: Id<ApplicationMarker>,
    /// Settings for the application's default in-app authorization, if enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_params: Option<InstallParams>,
    pub name: String,
    pub owner: Option<User>,
    pub primary_sku_id: Option<Id<OauthSkuMarker>>,
    /// URL of the application's privacy policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_policy_url: Option<String>,
    #[serde(default)]
    pub rpc_origins: Vec<String>,
    pub slug: Option<String>,
    /// Tags describing the content and functionality of the application.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    pub team: Option<Team>,
    /// URL of the application's terms of service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms_of_service_url: Option<String>,
    pub verify_key: String,
}

#[cfg(test)]
mod tests {
    use super::{Application, ApplicationFlags, Team, User};
    use crate::{id::Id, test::image_hash};
    use serde::{Deserialize, Serialize};
    use serde_test::Token;
    use static_assertions::{assert_fields, assert_impl_all};
    use std::{fmt::Debug, hash::Hash};

    assert_fields!(
        Application: bot_public,
        bot_require_code_grant,
        cover_image,
        custom_install_url,
        description,
        guild_id,
        flags,
        icon,
        id,
        install_params,
        name,
        owner,
        primary_sku_id,
        privacy_policy_url,
        rpc_origins,
        slug,
        tags,
        team,
        terms_of_service_url,
        verify_key
    );

    assert_impl_all!(
        Application: Clone,
        Debug,
        Deserialize<'static>,
        Eq,
        Hash,
        PartialEq,
        Serialize
    );

    #[allow(clippy::too_many_lines)]
    #[test]
    fn current_application_info() {
        let value = Application {
            bot_public: true,
            bot_require_code_grant: false,
            cover_image: Some(image_hash::COVER),
            custom_install_url: None,
            description: "a pretty cool application".to_owned(),
            guild_id: Some(Id::new(1)),
            flags: Some(ApplicationFlags::EMBEDDED),
            icon: Some(image_hash::ICON),
            id: Id::new(2),
            install_params: None,
            name: "cool application".to_owned(),
            owner: Some(User {
                accent_color: None,
                avatar: None,
                banner: None,
                bot: false,
                discriminator: 1,
                email: None,
                flags: None,
                id: Id::new(3),
                locale: None,
                mfa_enabled: None,
                name: "app dev".to_owned(),
                premium_type: None,
                public_flags: None,
                system: None,
                verified: None,
            }),
            primary_sku_id: Some(Id::new(4)),
            privacy_policy_url: Some("https://privacypolicy".into()),
            rpc_origins: vec!["one".to_owned()],
            slug: Some("app slug".to_owned()),
            tags: Some(Vec::from([
                "ponies".to_owned(),
                "horses".to_owned(),
                "friendship".to_owned(),
                "magic".to_owned(),
            ])),
            team: Some(Team {
                icon: None,
                id: Id::new(5),
                members: Vec::new(),
                name: "team name".into(),
                owner_user_id: Id::new(6),
            }),
            terms_of_service_url: Some("https://termsofservice".into()),
            verify_key: "key".to_owned(),
        };

        serde_test::assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "Application",
                    len: 18,
                },
                Token::Str("bot_public"),
                Token::Bool(true),
                Token::Str("bot_require_code_grant"),
                Token::Bool(false),
                Token::Str("cover_image"),
                Token::Some,
                Token::Str(image_hash::COVER_INPUT),
                Token::Str("description"),
                Token::Str("a pretty cool application"),
                Token::Str("guild_id"),
                Token::Some,
                Token::NewtypeStruct { name: "Id" },
                Token::Str("1"),
                Token::Str("flags"),
                Token::Some,
                Token::U64(131_072),
                Token::Str("icon"),
                Token::Some,
                Token::Str(image_hash::ICON_INPUT),
                Token::Str("id"),
                Token::NewtypeStruct { name: "Id" },
                Token::Str("2"),
                Token::Str("name"),
                Token::Str("cool application"),
                Token::Str("owner"),
                Token::Some,
                Token::Struct {
                    name: "User",
                    len: 7,
                },
                Token::Str("accent_color"),
                Token::None,
                Token::Str("avatar"),
                Token::None,
                Token::Str("banner"),
                Token::None,
                Token::Str("bot"),
                Token::Bool(false),
                Token::Str("discriminator"),
                Token::Str("0001"),
                Token::Str("id"),
                Token::NewtypeStruct { name: "Id" },
                Token::Str("3"),
                Token::Str("username"),
                Token::Str("app dev"),
                Token::StructEnd,
                Token::Str("primary_sku_id"),
                Token::Some,
                Token::NewtypeStruct { name: "Id" },
                Token::Str("4"),
                Token::Str("privacy_policy_url"),
                Token::Some,
                Token::Str("https://privacypolicy"),
                Token::Str("rpc_origins"),
                Token::Seq { len: Some(1) },
                Token::Str("one"),
                Token::SeqEnd,
                Token::Str("slug"),
                Token::Some,
                Token::Str("app slug"),
                Token::Str("tags"),
                Token::Some,
                Token::Seq { len: Some(4) },
                Token::Str("ponies"),
                Token::Str("horses"),
                Token::Str("friendship"),
                Token::Str("magic"),
                Token::SeqEnd,
                Token::Str("team"),
                Token::Some,
                Token::Struct {
                    name: "Team",
                    len: 5,
                },
                Token::Str("icon"),
                Token::None,
                Token::Str("id"),
                Token::NewtypeStruct { name: "Id" },
                Token::Str("5"),
                Token::Str("members"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("name"),
                Token::Str("team name"),
                Token::Str("owner_user_id"),
                Token::NewtypeStruct { name: "Id" },
                Token::Str("6"),
                Token::StructEnd,
                Token::Str("terms_of_service_url"),
                Token::Some,
                Token::Str("https://termsofservice"),
                Token::Str("verify_key"),
                Token::Str("key"),
                Token::StructEnd,
            ],
        );
    }
}
