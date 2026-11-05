use super::puma_boot_contract as contract;
use roundhouse::project::BuildTarget;

#[test]
fn puma_boot_hooks_refuse_unresolved_calls_in_callback_bodies() {
    let (_tree, errors) = contract::overlay()
        .write(
            "config/puma.rb",
            "after_booted { JSON.parse('{}').no_such_puma_boot_hook }\n",
        )
        .emit(BuildTarget::Ruby);
    assert!(
        errors
            .iter()
            .any(|error| error.contains("no_such_puma_boot_hook")),
        "{errors:?}"
    );
}

#[test]
fn puma_boot_hooks_report_targets_without_a_boot_event() {
    let (_tree, errors) = contract::overlay().emit(BuildTarget::Rust);
    assert!(
        errors
            .iter()
            .any(|error| error.contains("no server boot event")),
        "{errors:?}"
    );
}

#[test]
fn puma_boot_hooks_wait_for_the_boot_event_and_keep_registration_conditions() {
    let script = r#"
raise "hook ran while loading libraries" if ENV["PUMA_HOOK_EVENTS"]
ENV.delete("PUMA_HOOK_SELECTION")
require "puma"
require "puma/configuration"
config = Puma::Configuration.new
config.configure do |dsl|
  dsl.instance_eval(File.read("config/puma.rb"), "config/puma.rb")
end
raise "hook ran during registration" if ENV["PUMA_HOOK_EVENTS"]
config.events.fire_after_booted!
raise ENV["PUMA_HOOK_EVENTS"].inspect unless ENV["PUMA_HOOK_EVENTS"] == "first,second"
puts "puma boot hook contract passed"
"#;
    let run = contract::overlay().run_ruby(script);
    run.assert_passes();
    assert!(run.stdout.contains("puma boot hook contract passed"));
    let early_config = super::emit_and_run::ruby()
        .args(["-e", script])
        .current_dir(&run.emitted)
        .env("BLOG_DB", ":memory:")
        .output()
        .expect("load Puma config before the application");
    assert!(
        early_config.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&early_config.stdout),
        String::from_utf8_lossy(&early_config.stderr)
    );
}
