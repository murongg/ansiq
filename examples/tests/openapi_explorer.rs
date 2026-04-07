use ansiq_examples::scenarios::openapi_explorer::{OpenApiExplorerApp, rendered_screen_for_test};
use ansiq_runtime::Engine;
use ansiq_surface::Key;

fn fixture_spec() -> &'static str {
    r#"
openapi: 3.0.3
info:
  title: Demo API
  version: "1.0"
paths:
  /pets:
    get:
      tags: [pets]
      summary: List pets
      parameters:
        - in: query
          name: limit
          schema:
            type: integer
      responses:
        "200":
          description: ok
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Pets'
components:
  schemas:
    Pets:
      type: array
      items:
        $ref: '#/components/schemas/Pet'
    Pet:
      type: object
      properties:
        id:
          type: integer
        name:
          type: string
"#
}

#[test]
fn openapi_explorer_renders_navigation_operation_details_and_schema() {
    let mut engine = Engine::new(
        OpenApiExplorerApp::from_spec_text("fixture.yaml", fixture_spec())
            .expect("fixture should parse"),
    );

    engine.render_tree();
    let screen = rendered_screen_for_test(&engine, 140, 32);

    assert!(screen.contains("Demo API"));
    assert!(screen.contains("GET /pets"));
    assert!(screen.contains("List pets"));
    assert!(screen.contains("components/schemas/Pets"));
}

#[test]
fn openapi_explorer_switches_focus_and_updates_selected_operation() {
    let mut engine = Engine::new(
        OpenApiExplorerApp::from_spec_text("fixture.yaml", fixture_spec())
            .expect("fixture should parse"),
    );

    engine.render_tree();
    assert!(!engine.handle_input(Key::Tab));
    assert!(!engine.handle_input(Key::Down));
    engine.render_tree();

    let screen = rendered_screen_for_test(&engine, 140, 32);
    assert!(screen.contains("Operation"));
}
