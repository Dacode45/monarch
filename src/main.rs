use amethyst::ecs::SystemData;
use amethyst::prelude::*;

use monarch::*;

struct PongClient(Pong);

impl SimpleState for PongClient {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        data.world.register::<Paddle>();
        let handle = load_spritesheet(data.world);
        initialize_paddles(data.world, handle);
        initialize_camera(data.world);

        SimpleState::on_start(&mut self.0, data);
    }
}

fn load_spritesheet(
    world: &mut World,
) -> amethyst::assets::Handle<amethyst::renderer::SpriteSheet> {
    let texture_handle = {
        let loader = world.read_resource::<amethyst::assets::Loader>();
        let texture_storage =
            world.read_resource::<amethyst::assets::AssetStorage<amethyst::renderer::Texture>>();
        loader.load(
            "texture/pong_spritesheet.png",
            amethyst::renderer::ImageFormat::default(),
            (),
            &texture_storage,
        )
    };

    let loader = world.read_resource::<amethyst::assets::Loader>();
    let spritesheet_storage =
        world.read_resource::<amethyst::assets::AssetStorage<amethyst::renderer::SpriteSheet>>();
    loader.load(
        "texture/pong_spritesheet.ron",
        amethyst::renderer::SpriteSheetFormat(texture_handle),
        (),
        &spritesheet_storage,
    )
}

fn initialize_paddles(
    world: &mut amethyst::ecs::World,
    handle: amethyst::assets::Handle<amethyst::renderer::SpriteSheet>,
) {
    let mut left_transform = amethyst::core::transform::Transform::default();
    let mut right_transform = amethyst::core::transform::Transform::default();

    left_transform.set_translation_xyz(PADDLE_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 0.0);
    right_transform.set_translation_xyz(ARENA_WIDTH - PADDLE_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 0.0);

    let sprite_render = amethyst::renderer::SpriteRender {
        sprite_sheet: handle.clone(),
        sprite_number: 0,
    };

    world
        .create_entity()
        .with(left_transform)
        .with(Paddle::new(Side::Left))
        .with(sprite_render.clone())
        .build();
    world
        .create_entity()
        .with(right_transform)
        .with(Paddle::new(Side::Right))
        .with(sprite_render.clone())
        .build();
}

fn initialize_camera(world: &mut amethyst::ecs::World) {
    let mut transform = amethyst::core::transform::Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 1.0);

    world
        .create_entity()
        .with(amethyst::renderer::Camera::standard_2d(
            ARENA_WIDTH,
            ARENA_HEIGHT,
        ))
        .with(transform)
        .build();
}

#[derive(Debug, Default)]
struct ExampleGraph {
    dimensions: Option<amethyst::window::ScreenDimensions>,
    dirty: bool,
}

impl amethyst::renderer::GraphCreator<amethyst::renderer::types::DefaultBackend> for ExampleGraph {
    fn rebuild(&mut self, res: &amethyst::ecs::Resources) -> bool {
        let new_dimensions = res.try_fetch::<amethyst::window::ScreenDimensions>();
        use std::ops::Deref;
        if self.dimensions.as_ref() != new_dimensions.as_ref().map(|d| d.deref()) {
            self.dimensions = new_dimensions.map(|d| d.clone());
            self.dirty = true;
            return false;
        }
        return self.dirty;
    }

    fn builder(
        &mut self,
        factory: &mut amethyst::renderer::Factory<amethyst::renderer::types::DefaultBackend>,
        res: &amethyst::ecs::Resources,
    ) -> amethyst::renderer::GraphBuilder<
        amethyst::renderer::types::DefaultBackend,
        amethyst::ecs::Resources,
    > {
        self.dirty = false;

        let window = <amethyst::ecs::ReadExpect<'_, amethyst::window::Window>>::fetch(res);
        let dimensions = self.dimensions.as_ref().unwrap();
        let window_kind = amethyst::renderer::Kind::D2(
            dimensions.width() as u32,
            dimensions.height() as u32,
            1,
            1,
        );

        let surface = factory.create_surface(&window);
        let surface_format = factory.get_surface_format(&surface);

        let mut graph_builder = amethyst::renderer::GraphBuilder::new();
        let color = graph_builder.create_image(
            window_kind,
            1,
            surface_format,
            Some(amethyst::renderer::rendy::hal::command::ClearValue::Color(
                [0.0, 0.0, 0.0, 1.0].into(),
            )),
        );

        let depth = graph_builder.create_image(
            window_kind,
            1,
            amethyst::renderer::Format::D32Sfloat,
            Some(
                amethyst::renderer::rendy::hal::command::ClearValue::DepthStencil(
                    amethyst::renderer::rendy::hal::command::ClearDepthStencil(1.0, 0),
                ),
            ),
        );

        use amethyst::renderer::RenderGroupDesc;
        let pass = graph_builder.add_node(
            amethyst::renderer::SubpassBuilder::new()
                .with_group(amethyst::renderer::pass::DrawFlat2DDesc::new().builder())
                .with_color(color)
                .with_depth_stencil(depth)
                .into_pass(),
        );

        let _present = graph_builder.add_node(
            amethyst::renderer::rendy::graph::present::PresentNode::builder(
                factory, surface, color,
            )
            .with_dependency(pass),
        );

        graph_builder
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = amethyst::utils::application_root_dir()?;
    let display_config = app_root.join("resources").join("display_config.ron");
    let input_config = app_root.join("resources/bindings_config.ron");

    let input_bundle = amethyst::input::InputBundle::<amethyst::input::StringBindings>::new()
        .with_bindings_from_file(input_config)?;

    let game_data = GameDataBuilder::new()
        .with_bundle(amethyst::window::WindowBundle::from_config_path(
            display_config,
        ))?
        .with_bundle(input_bundle)?
        .with_bundle(amethyst::core::transform::TransformBundle::new())?
        .with(
            amethyst::assets::Processor::<amethyst::renderer::SpriteSheet>::new(),
            "sprite_sheet_processor",
            &[],
        )
        .with_thread_local(amethyst::renderer::RenderingSystem::<
            amethyst::renderer::types::DefaultBackend,
            _,
        >::new(ExampleGraph::default()));

    let asset_dir = app_root.join("assets");
    let mut game = Application::new(asset_dir, PongClient(Pong), game_data)?;
    game.run();
    Ok(())
}
