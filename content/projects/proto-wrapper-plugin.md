---
title: "Proto Wrapper Plugin"
slug: "proto-wrapper-plugin"
description: "Maven/Gradle plugin that generates version-agnostic Java wrappers from multiple protobuf schema versions"
date: "2025-01-25T10:00:00Z"
tags: ["java", "protobuf", "maven", "gradle", "code-generation"]
status: "active"
github_url: "https://github.com/alnovis/proto-wrapper-plugin"
featured: true
---

Proto Wrapper Plugin solves the challenge of working with multiple protobuf schema versions in long-lived systems by generating a unified Java API.

## Problem Solved

When protobuf schemas evolve across versions, you face type mismatches, version-specific code paths, and maintenance burden. This plugin generates wrapper interfaces that abstract away these differences.

## Features

- **Multi-version support**: Merge unlimited proto versions into unified API
- **Automatic type conflict handling**: INT_ENUM, WIDENING, STRING_BYTES resolution
- **Builder pattern**: Fluent API for creating/modifying messages
- **Well-known types**: Convert Timestamp, Duration to Java types
- **Incremental build**: 50%+ faster rebuilds
- **Embedded protoc**: No manual installation required
- **Java 8 compatibility**: Optional targeting

## Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Java 17 |
| Code Generation | JavaPoet |
| Proto Parsing | protobuf-java |
| Build Plugins | Maven Plugin API, Gradle |
| Testing | JUnit 5, AssertJ |

## Generated Output

```
com.example.model/
├── api/
│   ├── Order.java              # Interface
│   ├── VersionContext.java     # Factory
│   └── impl/AbstractOrder.java
├── v1/OrderV1.java             # Implementation
└── v2/OrderV2.java
```

## Usage Example

```java
// Works with any version
Order order = ctx.wrapOrder(anyVersionProto);
PaymentType type = order.getPaymentType();
byte[] bytes = order.toBytes();
```

Read the [detailed blog post](/blog/proto-wrapper-plugin) for architecture and implementation details.
