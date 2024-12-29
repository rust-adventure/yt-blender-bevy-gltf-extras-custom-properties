use avian3d::{prelude::*, PhysicsPlugins};
use bevy::{
    gltf::GltfMeshExtras, prelude::*,
    scene::SceneInstanceReady,
    time::common_conditions::on_timer,
};
use serde::{Deserialize, Serialize};
use std::{
    f32::consts::{FRAC_PI_4, PI},
    time::Duration,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ))
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                animate_light_direction,
                spawn_cubes.run_if(on_timer(
                    Duration::from_secs(1),
                )),
            ),
        )
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(10., 4., -5.0)
            .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
    ));

    commands.spawn((DirectionalLight {
        shadows_enabled: true,
        ..default()
    },));
    commands
        .spawn(SceneRoot(
            asset_server.load(
                GltfAssetLabel::Scene(1)
                    .from_asset("everything.glb"),
            ),
        ))
        .observe(on_scene_spawn);
}

fn on_scene_spawn(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    extras: Query<&GltfMeshExtras>,
) {
    for entity in
        children.iter_descendants(trigger.entity())
    {
        let Ok(gltf_mesh_extras) = extras.get(entity)
        else {
            continue;
        };
        let Ok(data) = serde_json::from_str::<BMeshExtras>(
            &gltf_mesh_extras.value,
        ) else {
            error!("couldn't deseralize extras");
            continue;
        };
        dbg!(&data);
        match data.collider {
            BCollider::TrimeshFromMesh => {
                commands.entity(entity).insert((
                    match data.rigid_body {
                        BRigidBody::Static => {
                            RigidBody::Static
                        }
                        BRigidBody::Dynamic => {
                            RigidBody::Dynamic
                        }
                    },
                    ColliderConstructor::TrimeshFromMesh,
                ));
            }
            BCollider::Cubiod => {
                let size = data.cube_size.expect(
                    "Cubiod collider must have cube_size",
                );
                commands.entity(entity).insert((
                    match data.rigid_body {
                        BRigidBody::Static => {
                            RigidBody::Static
                        }
                        BRigidBody::Dynamic => {
                            RigidBody::Dynamic
                        }
                    },
                    Collider::cuboid(
                        size.x, size.y, size.z,
                    ),
                ));
            }
        }
    }
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<
        &mut Transform,
        With<DirectionalLight>,
    >,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_secs() * PI / 5.0,
            -FRAC_PI_4,
        );
    }
}

fn spawn_cubes(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn((
            SceneRoot(
                asset_server.load(
                    GltfAssetLabel::Scene(0)
                        .from_asset("everything.glb"),
                ),
            ),
            Transform::from_xyz(0., 10., 0.),
        ))
        .observe(on_scene_spawn);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BMeshExtras {
    pub collider: BCollider,
    pub rigid_body: BRigidBody,
    pub cube_size: Option<Vec3>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BCollider {
    TrimeshFromMesh,
    Cubiod,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BRigidBody {
    Static,
    Dynamic,
}
