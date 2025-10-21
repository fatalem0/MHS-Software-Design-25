use command_producer::CommandProducer;

#[test]
fn test_produce_commands() {
    let mut producer = CommandProducer::new();
    let input = vec!["ls -la".to_string(), "echo hello".to_string()];

    let commands = producer.ProduceCommands(input);

    assert_eq!(commands.len(), 2);

    assert_eq!(commands[0].Name, "ls");
    assert_eq!(commands[0].Args, vec!["-la".to_string()]);
    assert_eq!(commands[0].Stdin, None);
    assert_eq!(commands[0].Stdout, None);

    assert_eq!(commands[1].Name, "echo");
    assert_eq!(commands[1].Args, vec!["hello".to_string()]);
    assert_eq!(commands[1].Stdin, None);
    assert_eq!(commands[1].Stdout, None);
}

#[test]
fn test_register_processors() {
    use std::collections::HashMap;
    use command_producer::CommandProcessorMap;

    let mut producer = CommandProducer::new();
    let processors: CommandProcessorMap = HashMap::new();

    producer.RegisterCmdProcessors(processors);
}
