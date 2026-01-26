use bevy::{color::palettes::basic::*, input_focus::InputFocus, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<InputFocus>()
        .add_systems(Startup, setup)
        .add_systems(Update, button_system)
        .run();
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn button_system(
    mut input_focus: ResMut<InputFocus>,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut Button,
            &Children,
        ),
        Changed<Interaction>,
    >,
    mut text_query: Query<&mut Text>,
) {
    for (entity, interaction, mut color, mut border_color, mut button, children) in
        &mut interaction_query
    {
        let mut text = text_query.get_mut(children[0]).unwrap();

        match *interaction {
            Interaction::Pressed => {
                input_focus.set(entity);
                **text = "Press".to_string();
                *color = PRESSED_BUTTON.into();
                *border_color = BorderColor::all(RED);

                // The accessibility system's only update the button's state when the `Button` component is marked as changed.
                button.set_changed();
            }
            Interaction::Hovered => {
                input_focus.set(entity);
                **text = "Hover".to_string();
                *color = HOVERED_BUTTON.into();
                *border_color = BorderColor::all(Color::WHITE);
                button.set_changed();
            }
            Interaction::None => {
                input_focus.clear();
                **text = "Button".to_string();
                *color = NORMAL_BUTTON.into();
                *border_color = BorderColor::all(Color::BLACK);
            }
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn(setup_ui());
}

fn setup_ui() -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(10.0),
            ..default()
        },
        children![
            (
                Button,
                Node {
                    width: px(150),
                    height: px(65),
                    border: UiRect::all(px(1)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::px(5., 5., 5., 5.),
                    ..default()
                },
                BorderColor::all(Color::WHITE),
                BackgroundColor(Color::BLACK),
                children![(
                    Text::new("Button"),
                    TextFont {
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                )]
            ),
            (
                Button,
                Node {
                    width: px(150),
                    height: px(65),
                    border: UiRect::all(px(1)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::px(5., 5., 5., 5.),
                    ..default()
                },
                BorderColor::all(Color::WHITE),
                BackgroundColor(Color::BLACK),
                children![(
                    Text::new("Button"),
                    TextFont {
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                )]
            )
        ],
    )
}
