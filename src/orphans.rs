//! Old code before the refactor that needs to be either cut out or worked into the new design
//!

#[derive(Bundle)]
pub struct Projectile {
    #[bundle]
    sprite_bundle: SpriteBundle,
    torque: AngularVelocity,
    collider: Collider,
    sensor: Sensor,
    events: ActiveEvents,
    collision_types: ActiveCollisionTypes,
    collision_groups: CollisionGroups,
    facing: Facing,
    velocity: LinearVelocity,
    attack: Attack,
    attack_timer: ProjectileLifetime,
}

impl Projectile {
    pub fn new(
        transform: &Transform,
        facing: &Facing,
        dir: Vec2,
        asset_server: &Res<AssetServer>,
    ) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                texture: asset_server.load("bottled_seaweed11x31.png"),
                transform: Transform::from_xyz(
                    transform.translation.x,
                    transform.translation.y,
                    ATTACK_LAYER,
                ),
                ..default()
            },
            torque: AngularVelocity::with_clockwise(THROW_ITEM_ROTATION_SPEED, !facing.is_left()),
            collider: Collider::cuboid(ATTACK_WIDTH / 2., ATTACK_HEIGHT / 2.),
            sensor: Sensor,
            events: ActiveEvents::COLLISION_EVENTS,
            collision_types: ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC,
            //TODO: define collision layer based on the fighter shooting projectile, load for asset files of fighter which "team" they are on
            collision_groups: CollisionGroups::new(BodyLayers::PLAYER_ATTACK, BodyLayers::ENEMY),
            facing: facing.clone(),
            velocity: LinearVelocity(dir * 300.), //TODO: Put the velocity in a cons,
            attack: Attack { damage: 10 },
            attack_timer: ProjectileLifetime(Timer::new(Duration::from_secs(1), false)),
        }
    }
}

#[derive(Bundle)]
pub struct ThrownItem {
    #[bundle]
    sprite_bundle: SpriteBundle,
    torque: AngularVelocity,
    // move_in_arc: MoveInArc,
    collider: Collider,
    sensor: Sensor,
    events: ActiveEvents,
    collision_types: ActiveCollisionTypes,
    collision_groups: CollisionGroups,
    attack: Attack,
}

impl ThrownItem {
    pub fn new(
        angles: (f32, f32),
        position: Vec2,
        facing: Facing,
        asset_server: &AssetServer,
    ) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                texture: asset_server.load("bottled_seaweed11x31.png"),
                transform: Transform::from_xyz(position.x, position.y, ITEM_LAYER),
                ..default()
            },
            torque: AngularVelocity::with_clockwise(
                consts::THROW_ITEM_ROTATION_SPEED,
                !facing.is_left(),
            ),
            move_in_arc: MoveInArc {
                //TODO: Set in consts
                radius: Vec2::new(
                    50.,
                    consts::PLAYER_HEIGHT + consts::THROW_ITEM_Y_OFFSET + consts::ITEM_HEIGHT,
                ),
                speed: consts::THROW_ITEM_SPEED,
                angle: angles.0,
                end_angle: angles.1,
                inverse_direction: facing.is_left(),
                origin: position,
            },
            collider: Collider::cuboid(ITEM_WIDTH / 2., ITEM_HEIGHT / 2.),
            sensor: Sensor,
            events: ActiveEvents::COLLISION_EVENTS,
            collision_types: ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC,
            //TODO: define collision layer based on the fighter throwing the item, load for asset files of fighter which "team" they are on
            collision_groups: CollisionGroups::new(BodyLayers::ITEM, BodyLayers::ENEMY),
            attack: Attack {
                damage: consts::THROW_ITEM_DAMAGE,
            },
        }
    }
}

fn player_projectile_attack(
    player_query: Query<(&Children, &Transform, &Facing, &ActionState<PlayerAction>), With<Player>>,
    items_meta_query: Query<&Handle<ItemMeta>>,
    items_meta: Res<Assets<ItemMeta>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (player_children, transform, facing, state, input) in player_query.iter() {
        if *state != State::Idle && *state != State::Running {
            continue;
        }

        let carried_item = item_carried_by_player(
            player_children,
            ITEM_BOTTLE_NAME,
            &items_meta_query,
            &items_meta,
        );

        if let Some(bottle_id) = carried_item {
            if input.just_pressed(PlayerAction::Shoot) {
                let mut dir = Vec2::X;

                if facing.is_left() {
                    dir = -dir;
                }

                let projectile = Projectile::new(transform, facing, dir, &asset_server);

                commands.spawn_bundle(projectile);

                commands.entity(bottle_id).despawn();
            }
        }
    }
}

fn player_throw(
    mut commands: Commands,
    player_query: Query<
        (
            &Children,
            &Transform,
            Option<&Facing>,
            &ActionState<PlayerAction>,
        ),
        With<Player>,
    >,
    items_meta_query: Query<&Handle<ItemMeta>>,
    items_meta: Res<Assets<ItemMeta>>,
    asset_server: Res<AssetServer>,
) {
    for (player_children, transform, facing_option, input) in player_query.iter() {
        let carried_item = item_carried_by_player(
            player_children,
            ITEM_BOTTLE_NAME,
            &items_meta_query,
            &items_meta,
        );

        if let Some(bottle_id) = carried_item {
            if input.just_pressed(PlayerAction::Throw) {
                let facing = match facing_option {
                    Some(f) => f.clone(),
                    None => Facing::Right,
                };

                let mut position = transform.translation.truncate();

                //Offset the position depending on the facing
                if facing.is_left() {
                    position.x -= consts::THROW_ITEM_X_OFFSET;
                } else {
                    position.x += consts::THROW_ITEM_X_OFFSET;
                }

                position.y -= consts::PLAYER_HEIGHT / 2.; //Set to the player feet

                let angles = match facing {
                    Facing::Left => (90. - consts::THROW_ITEM_ANGLE_OFFSET, 180.),
                    Facing::Right => (90. + consts::THROW_ITEM_ANGLE_OFFSET, 0.),
                };

                let thrown_item = ThrownItem::new(angles, position, facing, &asset_server);

                commands.spawn_bundle(thrown_item);

                commands.entity(bottle_id).despawn()
            }
        }
    }
}

fn projectile_cleanup(
    query: Query<(Entity, &ProjectileLifetime), With<Attack>>,
    mut commands: Commands,
) {
    for (entity, timer) in query.iter() {
        if timer.0.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn projectile_tick(mut query: Query<&mut ProjectileLifetime, With<Attack>>, time: Res<Time>) {
    for mut timer in query.iter_mut() {
        timer.0.tick(time.delta());
    }
}
