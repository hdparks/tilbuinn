use std::string;

use bevy::{prelude::*, ecs::{system::Command, query::Has, component::{self, Components}, entity::{MapEntities, Entities}, archetype::{self, Archetypes}}, sprite::collide_aabb::Collision, reflect::tuple_partial_eq, utils::petgraph::algo::tred::dag_transitive_reduction_closure, diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin}};
use rand::prelude::*;
use bevy_rapier2d::{prelude::*, rapier::crossbeam::epoch::Pointable};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0), RapierDebugRenderPlugin::default(), LogDiagnosticsPlugin::default(), FrameTimeDiagnosticsPlugin::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, (gravity, print_entities))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    for i in 0..1000 {
        commands.spawn((
            SpriteBundle {
                texture: asset_server.load("branding/icon.png"),
                transform: Transform::from_xyz(rand::random::<f32>() * 10000. - 5000.,rand::random::<f32>()* 10000. - 5000., 0.0).with_scale(Vec3::new(0.1,0.1,1.0)),
                ..default()
            }, 
            RigidBody::Dynamic,
            Gravity{},
            Velocity{
                linvel: Vec2::default(),
                angvel: 0.
            },
            Ccd::enabled(),
            ReadMassProperties::default(),
            ExternalImpulse::default()
        ))
            .insert(GravityScale(0.0))
            .insert(Collider::ball(100.))
            .insert(ColliderMassProperties::Mass(100.));
        
                }
}
fn print_entities(keyboard: Res<Input<KeyCode>>, all_entities: Query<Entity>, entities: &Entities, archetypes: &Archetypes, components: &Components){
    if keyboard.just_pressed(KeyCode::F1) {
        for entity in all_entities.iter(){
            println!("Entity {:?}", entity);
            if let Some(entity_location) = entities.get(entity){
                if let Some(archetype) = archetypes.get(entity_location.archetype_id) {
                    for component in archetype.components() {
                        if let Some(info) = components.get_info(component) {
                            println!("\t{}", info.name());
                        }
                    }
                }
            }
        }
    }
}

#[derive(Component)]
struct Gravity {}

static GRAVITY: f32 = 10000.0;

fn gravity(time: Res<Time>, mut query: Query<(&mut ExternalImpulse,&ReadMassProperties, &Transform, With<Gravity>)>) {
    // f = ma
    // a = g(m1+m2)/d^2
    let mut dv = vec![Vec2::new(0.,0.);query.iter().len()];
    for (i,(_, mass_a, transform_a, _)) in query.iter().enumerate() {
        for (j, (_, mass_b, transform_b, _)) in query.iter().enumerate(){
            if j <= i {continue;} 
            let dx = transform_a.translation.x - transform_b.translation.x;
            let dy = transform_a.translation.y - transform_b.translation.y;
            let dsq = dx.powi(2) + dy.powi(2);
            let d = dsq.sqrt();
            let f_g = GRAVITY * (mass_a.0.mass + mass_b.0.mass) / dsq * time.delta_seconds();
            let dv_local = Vec2::new(f_g * dx / d, f_g * dy / d);
            dv[i] -= dv_local;
            dv[j] += dv_local;
        }
    }

    for (i, (mut impulse, _, _, _)) in query.iter_mut().enumerate() {
        impulse.impulse += dv[i];
    }
}

