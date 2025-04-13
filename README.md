# Pixel-Perfect Outline Shader for Bevy

A easy-use pixel-perfect outline shader for bevy using vertex extrusion method. Inspired by [this wonderful tutorial](https://alexanderameye.github.io/notes/rendering-outlines).

## Features

- [x] Pixel perfect: the width of drawn outline is in pixel unit and the same as what we want.
- [x] Eliminate foreshortening: the width of outline is uniform from near view to far view.
- [x] Customizability. Width and color can be determined by user.
- [x] Integration with `bevy_mod_picking`.

## Usage

First, add `bevy_outline` as a dependency into your `Cargo.toml`:

```toml
[dependencies]
bevy_outline = "0.3"
```

Second, add `OutlinePlugin` into your app:

```rust, norun
App::new()
... ...
    .add_plugin(OutlinePlugin)
... ...
```

Third, use `OutlineMaterial` as a `MeshMaterial3d`, and use the marker component `OutlineRendered`:

```rust, norun
fn setup(
    ...
    mut outlines: ResMut<Assets<OutlineMaterial>>,
    ...
) {
    ...
    let outline_black = outlines.add(OutlineMaterial {
        width: 5.,
        color: Color::linear_rgba(0.0, 0.0, 0.0, 1.0).into(),
    });
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Cuboid::default()))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.8, 0.7, 0.8))),
        Transform::from_xyz(2.0, 0.5, 0.0),
        OutlineRendered,
        MeshMaterial3d(outline_black.clone()),
    ));
    ...
}
```

Note that the unit of `width` attribute of `OutlineMaterial` is **pixel**.

## Work with `bevy_mod_picking`

The initial motivation of this crate is to enable outlining instead of material substitution when a mesh is picked by `bevy_mod_picking`.
Use this functionality is very simple:
- enable the `picking` feature of this crate.
- add `picking::DefaultPickingPlugins` in **this** crate to your application.
- set the associated resource like `HoverOutline`, `SelectedOutline` and `PressedOutline` to enable the outlining when hovered, selected and pressed.
See [this example](https://github.com/YoshieraHuang/bevy_outline/tree/v0.1/examples/picking.rs) for demo.

## Demos

See [example folder](https://github.com/YoshieraHuang/bevy_outline/tree/v0.1/examples)

## Problems

- [x] ~~the width of outliner seems not to be uniform.~~
- [x] ~~outline of built-in torus seems weird (algorithm is wrong and will be fixed in 0.8)~~
- [x] ~~Pan + Orbit camera in example does not work with `main` branch~~

# Bevy Version Support

I intend to track the `main` branch of Bevy. PRs supporting this are welcome!

|bevy|bevy_outline|
|---|---|
|0.15.3|0.1|

# License

This project is licensed under the MIT License.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in bevy_outline by you, shall be licensed as MIT, without any additional terms or conditions.
