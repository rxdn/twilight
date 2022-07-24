//! Types for Message [`Component`] support.
//!
//! Message components are a Discord API framework for adding interactive
//! elements to created [`Message`]s.
//!
//! Refer to [Discord Docs/Message Components] for additional information.
//!
//! [`Message`]: crate::channel::Message
//! [Discord Docs/Message Components]: https://discord.com/developers/docs/interactions/message-components

pub mod action_row;
pub mod button;
pub mod select_menu;
pub mod text_input;

mod kind;

pub use self::{
    action_row::ActionRow, button::Button, kind::ComponentType, select_menu::SelectMenu,
    text_input::TextInput,
};

use crate::{application::component::select_menu::SelectMenuOption, channel::ReactionType};
use serde::{
    de::{Deserializer, Error as DeError, IgnoredAny, MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize, Serializer,
};
use serde_value::{DeserializerError, Value};
use std::fmt::{Formatter, Result as FmtResult};

/// Interactive element of a message that an application uses.
///
/// Refer to [Discord Docs/Message Components] for additional information.
///
/// [Discord Docs/Message Components]: https://discord.com/developers/docs/interactions/message-components#what-are-components
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Component {
    ActionRow(ActionRow),
    Button(Button),
    SelectMenu(SelectMenu),
    TextInput(TextInput),
    Unknown(u8),
}

impl Component {
    /// Type of component that this is.
    ///
    /// ```
    /// use twilight_model::application::component::{
    ///     button::{Button, ButtonStyle},
    ///     Component, ComponentType,
    /// };
    ///
    /// let component = Component::Button(Button {
    ///     custom_id: None,
    ///     disabled: false,
    ///     emoji: None,
    ///     label: Some("ping".to_owned()),
    ///     style: ButtonStyle::Primary,
    ///     url: None,
    /// });
    ///
    /// assert_eq!(ComponentType::Button, component.kind());
    /// ```
    pub const fn kind(&self) -> ComponentType {
        match self {
            Self::ActionRow(_) => ComponentType::ActionRow,
            Self::Button(_) => ComponentType::Button,
            Self::SelectMenu(_) => ComponentType::SelectMenu,
            Self::TextInput(_) => ComponentType::TextInput,
            Component::Unknown(unknown) => ComponentType::Unknown(*unknown),
        }
    }
}

impl From<ActionRow> for Component {
    fn from(action_row: ActionRow) -> Self {
        Self::ActionRow(action_row)
    }
}

impl From<Button> for Component {
    fn from(button: Button) -> Self {
        Self::Button(button)
    }
}

impl From<SelectMenu> for Component {
    fn from(select_menu: SelectMenu) -> Self {
        Self::SelectMenu(select_menu)
    }
}

impl From<TextInput> for Component {
    fn from(text_input: TextInput) -> Self {
        Self::TextInput(text_input)
    }
}

impl<'de> Deserialize<'de> for Component {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(ComponentVisitor)
    }
}

#[derive(Debug, Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum Field {
    Components,
    CustomId,
    Disabled,
    Emoji,
    Label,
    MaxLength,
    MaxValues,
    MinLength,
    MinValues,
    Options,
    Placeholder,
    Required,
    Style,
    Type,
    Url,
    Value,
}

struct ComponentVisitor;

impl<'de> Visitor<'de> for ComponentVisitor {
    type Value = Component;

    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("struct Component")
    }

    #[allow(clippy::too_many_lines)]
    fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
        // Required fields.
        let mut components: Option<Vec<Component>> = None;
        let mut kind: Option<ComponentType> = None;
        let mut options: Option<Vec<SelectMenuOption>> = None;
        let mut style: Option<Value> = None;

        // Liminal fields.
        let mut custom_id: Option<Option<Value>> = None;
        let mut label: Option<Option<String>> = None;

        // Optional fields.
        let mut disabled: Option<bool> = None;
        let mut emoji: Option<Option<ReactionType>> = None;
        let mut max_length: Option<Option<u16>> = None;
        let mut max_values: Option<Option<u8>> = None;
        let mut min_length: Option<Option<u16>> = None;
        let mut min_values: Option<Option<u8>> = None;
        let mut placeholder: Option<Option<String>> = None;
        let mut required: Option<Option<bool>> = None;
        let mut url: Option<Option<String>> = None;
        let mut value: Option<Option<String>> = None;

        let span = tracing::trace_span!("deserializing component");
        let _span_enter = span.enter();

        loop {
            let span_child = tracing::trace_span!("iterating over element");
            let _span_child_enter = span_child.enter();

            let key = match map.next_key() {
                Ok(Some(key)) => {
                    tracing::trace!(?key, "found key");

                    key
                }
                Ok(None) => break,
                Err(why) => {
                    // Encountered when we run into an unknown key.
                    map.next_value::<IgnoredAny>()?;

                    tracing::trace!("ran into an unknown key: {why:?}");

                    continue;
                }
            };

            match key {
                Field::Components => {
                    if components.is_some() {
                        return Err(DeError::duplicate_field("components"));
                    }

                    components = Some(map.next_value()?);
                }
                Field::CustomId => {
                    if custom_id.is_some() {
                        return Err(DeError::duplicate_field("custom_id"));
                    }

                    custom_id = Some(map.next_value()?);
                }
                Field::Disabled => {
                    if disabled.is_some() {
                        return Err(DeError::duplicate_field("disabled"));
                    }

                    disabled = Some(map.next_value()?);
                }
                Field::Emoji => {
                    if emoji.is_some() {
                        return Err(DeError::duplicate_field("emoji"));
                    }

                    emoji = Some(map.next_value()?);
                }
                Field::Label => {
                    if label.is_some() {
                        return Err(DeError::duplicate_field("label"));
                    }

                    label = Some(map.next_value()?);
                }
                Field::MaxLength => {
                    if max_length.is_some() {
                        return Err(DeError::duplicate_field("max_length"));
                    }

                    max_length = Some(map.next_value()?);
                }
                Field::MaxValues => {
                    if max_values.is_some() {
                        return Err(DeError::duplicate_field("max_values"));
                    }

                    max_values = Some(map.next_value()?);
                }
                Field::MinLength => {
                    if min_length.is_some() {
                        return Err(DeError::duplicate_field("min_length"));
                    }

                    min_length = Some(map.next_value()?);
                }
                Field::MinValues => {
                    if min_values.is_some() {
                        return Err(DeError::duplicate_field("min_values"));
                    }

                    min_values = Some(map.next_value()?);
                }
                Field::Options => {
                    if options.is_some() {
                        return Err(DeError::duplicate_field("options"));
                    }

                    options = Some(map.next_value()?);
                }
                Field::Placeholder => {
                    if placeholder.is_some() {
                        return Err(DeError::duplicate_field("placeholder"));
                    }

                    placeholder = Some(map.next_value()?);
                }
                Field::Required => {
                    if required.is_some() {
                        return Err(DeError::duplicate_field("required"));
                    }

                    required = Some(map.next_value()?);
                }
                Field::Style => {
                    if style.is_some() {
                        return Err(DeError::duplicate_field("style"));
                    }

                    style = Some(map.next_value()?);
                }
                Field::Type => {
                    if kind.is_some() {
                        return Err(DeError::duplicate_field("type"));
                    }

                    kind = Some(map.next_value()?);
                }
                Field::Url => {
                    if url.is_some() {
                        return Err(DeError::duplicate_field("url"));
                    }

                    url = Some(map.next_value()?);
                }
                Field::Value => {
                    if value.is_some() {
                        return Err(DeError::duplicate_field("value"));
                    }

                    value = Some(map.next_value()?);
                }
            };
        }

        tracing::trace!(
            ?components,
            ?custom_id,
            ?disabled,
            ?emoji,
            ?label,
            ?max_length,
            ?max_values,
            ?min_length,
            ?min_values,
            ?options,
            ?placeholder,
            ?required,
            ?style,
            ?kind,
            ?url,
            ?value
        );

        let kind = kind.ok_or_else(|| DeError::missing_field("type"))?;

        Ok(match kind {
            // Required fields:
            // - components
            ComponentType::ActionRow => {
                let components = components.ok_or_else(|| DeError::missing_field("components"))?;

                Self::Value::ActionRow(ActionRow { components })
            }
            // Required fields:
            // - style
            //
            // Optional fields:
            // - custom_id
            // - disabled
            // - emoji
            // - label
            // - url
            ComponentType::Button => {
                let style = style
                    .ok_or_else(|| DeError::missing_field("style"))?
                    .deserialize_into()
                    .map_err(DeserializerError::into_error)?;

                let custom_id = custom_id
                    .flatten()
                    .map(Value::deserialize_into)
                    .transpose()
                    .map_err(DeserializerError::into_error)?;

                Self::Value::Button(Button {
                    custom_id,
                    disabled: disabled.unwrap_or_default(),
                    emoji: emoji.unwrap_or_default(),
                    label: label.flatten(),
                    style,
                    url: url.unwrap_or_default(),
                })
            }
            // Required fields:
            // - custom_id
            // - options
            //
            // Optional fields:
            // - disabled
            // - max_values
            // - min_values
            // - placeholder
            ComponentType::SelectMenu => {
                let custom_id = custom_id
                    .flatten()
                    .ok_or_else(|| DeError::missing_field("custom_id"))?
                    .deserialize_into()
                    .map_err(DeserializerError::into_error)?;

                let options = options.ok_or_else(|| DeError::missing_field("options"))?;

                Self::Value::SelectMenu(SelectMenu {
                    custom_id,
                    disabled: disabled.unwrap_or_default(),
                    max_values: max_values.unwrap_or_default(),
                    min_values: min_values.unwrap_or_default(),
                    options,
                    placeholder: placeholder.unwrap_or_default(),
                })
            }
            // Required fields:
            // - custom_id
            // - label
            // - style
            //
            // Optional fields:
            // - max_length
            // - min_length
            // - placeholder
            // - required
            // - value
            ComponentType::TextInput => {
                let custom_id = custom_id
                    .flatten()
                    .ok_or_else(|| DeError::missing_field("custom_id"))?
                    .deserialize_into()
                    .map_err(DeserializerError::into_error)?;

                let label = label
                    .flatten()
                    .ok_or_else(|| DeError::missing_field("label"))?;

                let style = style
                    .ok_or_else(|| DeError::missing_field("style"))?
                    .deserialize_into()
                    .map_err(DeserializerError::into_error)?;

                Self::Value::TextInput(TextInput {
                    custom_id,
                    label,
                    max_length: max_length.unwrap_or_default(),
                    min_length: min_length.unwrap_or_default(),
                    placeholder: placeholder.unwrap_or_default(),
                    required: required.unwrap_or_default(),
                    style,
                    value: value.unwrap_or_default(),
                })
            }
            ComponentType::Unknown(unknown) => Self::Value::Unknown(unknown),
        })
    }
}

impl Serialize for Component {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let len = match self {
            // Required fields:
            // - type
            // - components
            Component::ActionRow(_) => 2,
            // Required fields:
            // - type
            // - style
            //
            // Optional fields:
            // - custom_id
            // - disabled
            // - emoji
            // - label
            // - url
            Component::Button(button) => {
                2 + usize::from(button.custom_id.is_some())
                    + usize::from(button.disabled)
                    + usize::from(button.emoji.is_some())
                    + usize::from(button.label.is_some())
                    + usize::from(button.url.is_some())
            }
            // Required fields:
            // - custom_id
            // - options
            // - type
            //
            // Optional fields:
            // - disabled
            // - max_values
            // - min_values
            // - placeholder
            Component::SelectMenu(select_menu) => {
                3 + usize::from(select_menu.disabled)
                    + usize::from(select_menu.max_values.is_some())
                    + usize::from(select_menu.min_values.is_some())
                    + usize::from(select_menu.placeholder.is_some())
            }
            // Required fields:
            // - custom_id
            // - label
            // - style
            // - type
            //
            // Optional fields:
            // - max_length
            // - min_length
            // - placeholder
            // - required
            // - value
            Component::TextInput(text_input) => {
                4 + usize::from(text_input.max_length.is_some())
                    + usize::from(text_input.min_length.is_some())
                    + usize::from(text_input.placeholder.is_some())
                    + usize::from(text_input.required.is_some())
                    + usize::from(text_input.value.is_some())
            }
            // We are dropping fields here but nothing we can do about that for
            // the time being.
            Component::Unknown(_) => 1,
        };

        let mut state = serializer.serialize_struct("Component", len)?;

        match self {
            Component::ActionRow(action_row) => {
                state.serialize_field("type", &ComponentType::ActionRow)?;

                state.serialize_field("components", &action_row.components)?;
            }
            Component::Button(button) => {
                state.serialize_field("type", &ComponentType::Button)?;

                if button.custom_id.is_some() {
                    state.serialize_field("custom_id", &button.custom_id)?;
                }

                if button.disabled {
                    state.serialize_field("disabled", &button.disabled)?;
                }

                if button.emoji.is_some() {
                    state.serialize_field("emoji", &button.emoji)?;
                }

                if button.label.is_some() {
                    state.serialize_field("label", &button.label)?;
                }

                state.serialize_field("style", &button.style)?;

                if button.url.is_some() {
                    state.serialize_field("url", &button.url)?;
                }
            }
            Component::SelectMenu(select_menu) => {
                state.serialize_field("type", &ComponentType::SelectMenu)?;

                // Due to `custom_id` being required in some variants and
                // optional in others, serialize as an Option.
                state.serialize_field("custom_id", &Some(&select_menu.custom_id))?;

                state.serialize_field("disabled", &select_menu.disabled)?;

                if select_menu.max_values.is_some() {
                    state.serialize_field("max_values", &select_menu.max_values)?;
                }

                if select_menu.min_values.is_some() {
                    state.serialize_field("min_values", &select_menu.min_values)?;
                }

                state.serialize_field("options", &select_menu.options)?;

                if select_menu.placeholder.is_some() {
                    state.serialize_field("placeholder", &select_menu.placeholder)?;
                }
            }
            Component::TextInput(text_input) => {
                state.serialize_field("type", &ComponentType::TextInput)?;

                // Due to `custom_id` and `label` being required in some
                // variants and optional in others, serialize as an Option.
                state.serialize_field("custom_id", &Some(&text_input.custom_id))?;
                state.serialize_field("label", &Some(&text_input.label))?;

                if text_input.max_length.is_some() {
                    state.serialize_field("max_length", &text_input.max_length)?;
                }

                if text_input.min_length.is_some() {
                    state.serialize_field("min_length", &text_input.min_length)?;
                }

                if text_input.placeholder.is_some() {
                    state.serialize_field("placeholder", &text_input.placeholder)?;
                }

                if text_input.required.is_some() {
                    state.serialize_field("required", &text_input.required)?;
                }

                state.serialize_field("style", &text_input.style)?;

                if text_input.value.is_some() {
                    state.serialize_field("value", &text_input.value)?;
                }
            }
            // We are not serializing all fields so this will fail to
            // deserialize. But it is all that can be done to avoid losing
            // incoming messages at this time.
            Component::Unknown(unknown) => {
                state.serialize_field("type", &ComponentType::Unknown(*unknown))?;
            }
        }

        state.end()
    }
}

#[cfg(test)]
mod tests {
    // Required due to the use of a unicode emoji in a constant.
    #![allow(clippy::non_ascii_literal)]

    use super::*;
    use crate::application::component::{
        button::ButtonStyle, select_menu::SelectMenuOption, text_input::TextInputStyle,
    };
    use serde_test::Token;
    use static_assertions::assert_impl_all;

    assert_impl_all!(
        Component: From<ActionRow>,
        From<Button>,
        From<SelectMenu>,
        From<TextInput>
    );

    #[allow(clippy::too_many_lines)]
    #[test]
    fn component_full() {
        let component = Component::ActionRow(ActionRow {
            components: Vec::from([
                Component::Button(Button {
                    custom_id: Some("test custom id".into()),
                    disabled: true,
                    emoji: None,
                    label: Some("test label".into()),
                    style: ButtonStyle::Primary,
                    url: None,
                }),
                Component::SelectMenu(SelectMenu {
                    custom_id: "test custom id 2".into(),
                    disabled: false,
                    max_values: Some(25),
                    min_values: Some(5),
                    options: Vec::from([SelectMenuOption {
                        label: "test option label".into(),
                        value: "test option value".into(),
                        description: Some("test description".into()),
                        emoji: None,
                        default: false,
                    }]),
                    placeholder: Some("test placeholder".into()),
                }),
            ]),
        });

        serde_test::assert_tokens(
            &component,
            &[
                Token::Struct {
                    name: "Component",
                    len: 2,
                },
                Token::Str("type"),
                Token::U8(ComponentType::ActionRow.into()),
                Token::Str("components"),
                Token::Seq { len: Some(2) },
                Token::Struct {
                    name: "Component",
                    len: 5,
                },
                Token::Str("type"),
                Token::U8(ComponentType::Button.into()),
                Token::Str("custom_id"),
                Token::Some,
                Token::Str("test custom id"),
                Token::Str("disabled"),
                Token::Bool(true),
                Token::Str("label"),
                Token::Some,
                Token::Str("test label"),
                Token::Str("style"),
                Token::U8(ButtonStyle::Primary.into()),
                Token::StructEnd,
                Token::Struct {
                    name: "Component",
                    len: 6,
                },
                Token::Str("type"),
                Token::U8(ComponentType::SelectMenu.into()),
                Token::Str("custom_id"),
                Token::Some,
                Token::Str("test custom id 2"),
                Token::Str("disabled"),
                Token::Bool(false),
                Token::Str("max_values"),
                Token::Some,
                Token::U8(25),
                Token::Str("min_values"),
                Token::Some,
                Token::U8(5),
                Token::Str("options"),
                Token::Seq { len: Some(1) },
                Token::Struct {
                    name: "SelectMenuOption",
                    len: 4,
                },
                Token::Str("default"),
                Token::Bool(false),
                Token::Str("description"),
                Token::Some,
                Token::Str("test description"),
                Token::Str("label"),
                Token::Str("test option label"),
                Token::Str("value"),
                Token::Str("test option value"),
                Token::StructEnd,
                Token::SeqEnd,
                Token::Str("placeholder"),
                Token::Some,
                Token::Str("test placeholder"),
                Token::StructEnd,
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn action_row() {
        let value = Component::ActionRow(ActionRow {
            components: Vec::from([Component::Button(Button {
                custom_id: Some("button-1".to_owned()),
                disabled: false,
                emoji: None,
                style: ButtonStyle::Primary,
                label: Some("Button".to_owned()),
                url: None,
            })]),
        });

        serde_test::assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "Component",
                    len: 2,
                },
                Token::String("type"),
                Token::U8(ComponentType::ActionRow.into()),
                Token::String("components"),
                Token::Seq { len: Some(1) },
                Token::Struct {
                    name: "Component",
                    len: 4,
                },
                Token::String("type"),
                Token::U8(2),
                Token::String("custom_id"),
                Token::Some,
                Token::String("button-1"),
                Token::String("label"),
                Token::Some,
                Token::String("Button"),
                Token::String("style"),
                Token::U8(1),
                Token::StructEnd,
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn button() {
        // Free Palestine.
        //
        // Palestinian Flag.
        const FLAG: &str = "🇵🇸";

        let value = Component::Button(Button {
            custom_id: Some("test".to_owned()),
            disabled: false,
            emoji: Some(ReactionType::Unicode {
                name: FLAG.to_owned(),
            }),
            label: Some("Test".to_owned()),
            style: ButtonStyle::Link,
            url: Some("https://twilight.rs".to_owned()),
        });

        serde_test::assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "Component",
                    len: 6,
                },
                Token::String("type"),
                Token::U8(ComponentType::Button.into()),
                Token::String("custom_id"),
                Token::Some,
                Token::String("test"),
                Token::String("emoji"),
                Token::Some,
                Token::Struct {
                    name: "ReactionType",
                    len: 1,
                },
                Token::String("name"),
                Token::String(FLAG),
                Token::StructEnd,
                Token::String("label"),
                Token::Some,
                Token::String("Test"),
                Token::String("style"),
                Token::U8(ButtonStyle::Link.into()),
                Token::String("url"),
                Token::Some,
                Token::String("https://twilight.rs"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn text_input() {
        let value = Component::TextInput(TextInput {
            custom_id: "test".to_owned(),
            label: "The label".to_owned(),
            max_length: Some(100),
            min_length: Some(1),
            placeholder: Some("Taking this place".to_owned()),
            required: Some(true),
            style: TextInputStyle::Short,
            value: Some("Hello World!".to_owned()),
        });

        serde_test::assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "Component",
                    len: 9,
                },
                Token::String("type"),
                Token::U8(ComponentType::TextInput.into()),
                Token::String("custom_id"),
                Token::Some,
                Token::String("test"),
                Token::String("label"),
                Token::Some,
                Token::String("The label"),
                Token::String("max_length"),
                Token::Some,
                Token::U16(100),
                Token::String("min_length"),
                Token::Some,
                Token::U16(1),
                Token::String("placeholder"),
                Token::Some,
                Token::String("Taking this place"),
                Token::String("required"),
                Token::Some,
                Token::Bool(true),
                Token::String("style"),
                Token::U8(TextInputStyle::Short as u8),
                Token::String("value"),
                Token::Some,
                Token::String("Hello World!"),
                Token::StructEnd,
            ],
        );
    }
}
