use std::io::Write;

use zircon_runtime::core::runtime::tasks::{TaskPool, TaskPoolDescriptor};

use super::*;

#[test]
fn single_worker_join_reads_capture_and_polls_without_blocking() {
    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
    let (mut stdout_writer, mut stderr_writer, mut readers) =
        create_output_capture("single-worker contract").expect("capture should open");
    stdout_writer.write_all(b"stdout\n").expect("stdout write");
    stderr_writer.write_all(b"stderr\n").expect("stderr write");

    let (output, polled) = join_output_with_poll(&pool, &mut readers, || 42_u32);

    let output = output.expect("capture reads should succeed");
    assert_eq!(output.stdout, b"stdout\n");
    assert_eq!(output.stderr, b"stderr\n");
    assert_eq!(polled, 42);
}

#[test]
fn capture_reader_yields_at_the_byte_budget_while_output_remains() {
    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
    let (mut stdout_writer, _stderr_writer, mut readers) =
        create_output_capture("bounded capture contract").expect("capture should open");
    let output = vec![b'x'; OUTPUT_CAPTURE_READ_CHUNK_BYTES as usize * 2 + 7];
    stdout_writer.write_all(&output).expect("stdout write");

    let (first, first_poll) = join_output_with_poll(&pool, &mut readers, || 1_u32);
    let (second, second_poll) = join_output_with_poll(&pool, &mut readers, || 2_u32);

    assert_eq!(
        first.expect("first capture read").stdout.len(),
        OUTPUT_CAPTURE_READ_CHUNK_BYTES as usize
    );
    assert_eq!(
        second.expect("second capture read").stdout.len(),
        OUTPUT_CAPTURE_READ_CHUNK_BYTES as usize
    );
    assert_eq!((first_poll, second_poll), (1, 2));
}
