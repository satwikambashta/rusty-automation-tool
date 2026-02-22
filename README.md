# Rusty Automation Tool

`rusty-automation-tool` is a high-performance, event-driven workflow automation engine written in Rust. The project aims to provide a fast and scalable architecture capable of executing DAG-based workflows, with native support for plugin-based integrations and built-in AI nodes.

## Current State

This project is currently in the initial scaffolding stage. The basic workspace structure is defined and divided into several dedicated crates handling the API, core engine, workflow nodes, execution queue, and persistence layer.

## Future Roadmap

The tool is being built to support advanced automation and agentic execution capabilities. Below is an outline of features and systems planned for the future:

### Planned Features (Phase 1 & MVP)
- **Workflow Execution Engine**: Robust DAG executor with topological sorting for deterministic workflows.
- **REST API + Dashboard**: An initial interface to submit and monitor workflow executions.
- **Built-in Node Implementations**: Including HTTP requests, data transformation, conditional logic, and AI chat integration (OpenAI-compatible).
- **Triggers**: Support for webhooks, cron-based scheduling, and manual triggers.
- **Database Backend**: PostgreSQL mapping via `sqlx` to store workflows, triggers, executions, and secure secrets management.

### Next Generation Features (Phase 2 & Beyond)
As the project evolves, the following extensions are planned:
- **WASM Plugin Ecosystem**: Move beyond static nodes into dynamically loaded WASM plugins using `wasmtime`, enabling community-driven plugins in a secure sandbox.
- **Agentic Workflows**: Implementation of an "Agent loop" node for complex, autonomous reasoning steps, paired with vector database integration for knowledge retrieval.
- **Distributed Execution**: Scaling out the queue implementation from a basic Postgres-driven queue to Redis and potentially event streaming (Kafka/NATS) for horizontal distributed execution.
- **Advanced Architecture Elements**: Multi-tenant architecture, versioned workflow definitions, and a comprehensive SaaS control plane.
- **WASM Marketplace**: A plugin marketplace for developers to publish and share specific workflow nodes and plugins.

Stay tuned as we bring these components online!
