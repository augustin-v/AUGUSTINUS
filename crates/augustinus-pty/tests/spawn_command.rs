#[cfg(unix)]
mod unix {
    use std::time::{Duration, Instant};

    use augustinus_pty::PtySession;

    #[test]
    fn spawn_command_echo_smoke() {
        let mut session =
            PtySession::spawn_command("/bin/echo", &["hello-spawn-command"], 80, 24).unwrap();

        let deadline = Instant::now() + Duration::from_millis(500);
        loop {
            session.poll();
            let snapshot = session.snapshot();
            if snapshot.contents.contains("hello-spawn-command") {
                return;
            }
            if Instant::now() >= deadline {
                panic!("expected output not found; snapshot:\n{}", snapshot.contents);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

