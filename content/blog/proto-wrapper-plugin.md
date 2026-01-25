---
title: "Proto Wrapper Plugin: Version-Agnostic Protobuf Wrappers"
slug: "proto-wrapper-plugin"
description: "How to handle multiple protobuf schema versions with a unified Java API using automatic type conflict resolution"
date: "2025-01-25T10:00:00Z"
tags: ["java", "protobuf", "maven", "gradle", "code-generation"]
draft: false
---

When working with long-lived systems that process protobuf messages, schema evolution becomes a significant challenge. Fields get renamed, types change from `int32` to `enum`, new versions add required fields. Proto Wrapper Plugin solves this by generating a unified Java API that abstracts away version differences.

## The Problem

In systems processing multiple protocol versions, you typically face:

- **Type mismatches**: A field is `int32` in v1 but becomes `enum` in v2
- **Version-specific code paths**: Different handlers for each version
- **Maintenance burden**: Changes ripple through the codebase

Traditional approach requires separate code for each version:

```java
// Without Proto Wrapper - version-specific handling everywhere
if (version == 1) {
    int paymentType = requestV1.getPaymentType();
    processPayment(paymentType);
} else if (version == 2) {
    PaymentType type = requestV2.getPaymentType();
    processPayment(type.getNumber());
}
```

## The Solution

Proto Wrapper generates unified interfaces that work across all versions:

```java
// With Proto Wrapper - single code path
Order order = versionContext.wrapOrder(anyVersionProto);

// Type conflicts handled automatically
PaymentType type = order.getPaymentType();  // Works with int or enum
long amount = order.getTotalAmount();        // Auto-widened from int32/int64

// Serialize back to original version
byte[] bytes = order.toBytes();
```

## Quick Start

### Maven

```xml
<plugin>
    <groupId>io.alnovis</groupId>
    <artifactId>proto-wrapper-maven-plugin</artifactId>
    <version>2.3.0</version>
    <configuration>
        <basePackage>com.example.model</basePackage>
        <protoRoot>${basedir}/proto</protoRoot>
        <versions>
            <version><protoDir>v1</protoDir></version>
            <version><protoDir>v2</protoDir></version>
        </versions>
    </configuration>
    <executions>
        <execution><goals><goal>generate</goal></goals></execution>
    </executions>
</plugin>
```

Run with `mvn generate-sources`.

### Gradle

```kotlin
plugins {
    id("io.alnovis.proto-wrapper") version "2.3.0"
}

protoWrapper {
    basePackage.set("com.example.model")
    protoRoot.set(file("proto"))
    versions {
        version("v1")
        version("v2")
    }
}
```

Run with `./gradlew generateProtoWrapper`.

## Type Conflict Handling

The plugin automatically detects and resolves type conflicts between versions:

| Conflict | Example | Resolution |
|----------|---------|------------|
| INT_ENUM | `int32` to `enum` | Dual getters: `getType()` + `getTypeEnum()` |
| WIDENING | `int32` to `int64` | Unified as wider type with validation |
| STRING_BYTES | `string` to `bytes` | Dual getters: `getText()` + `getTextBytes()` |
| PRIMITIVE_MESSAGE | `int32` to `Money` | Runtime support checks |

## Key Features

- **Multi-version support**: Unlimited proto versions in single API
- **Builder pattern**: Fluent API for creating/modifying messages
- **Well-known types**: Automatic conversion (Timestamp to Instant, etc.)
- **Oneof handling**: Full support with conflict detection
- **Incremental build**: 50%+ faster rebuilds for unchanged protos
- **Embedded protoc**: No manual installation required
- **Java 8 compatibility**: Configure with `targetJavaVersion=8`

## Generated Code Structure

```
target/generated-sources/proto-wrapper/
└── com/example/model/
    ├── api/
    │   ├── Order.java              # Interface
    │   ├── PaymentType.java        # Unified enum
    │   ├── VersionContext.java     # Factory interface
    │   └── impl/
    │       └── AbstractOrder.java  # Template methods
    ├── v1/
    │   ├── OrderV1.java            # V1 implementation
    │   └── VersionContextV1.java
    └── v2/
        ├── OrderV2.java            # V2 implementation
        └── VersionContextV2.java
```

## How It Works

1. **Schema Analysis**: Plugin parses all proto files from each version
2. **Field Merging**: Fields with same name/number are merged, conflicts detected
3. **Type Resolution**: Conflict resolution strategies applied automatically
4. **Code Generation**: JavaPoet generates interfaces, implementations, builders

The generated code is plain Java with no runtime dependencies beyond protobuf itself.

## Schema Diff Tool

Compare schemas between versions to detect breaking changes:

```bash
mvn proto-wrapper:diff -Dfrom=v1 -Dto=v2
```

Output shows added/removed fields, type changes, and compatibility warnings.

## Links

- [GitHub Repository](https://github.com/alnovis/proto-wrapper-plugin)
- [Maven Central](https://central.sonatype.com/artifact/io.alnovis/proto-wrapper-core)
- [Full Documentation](https://github.com/alnovis/proto-wrapper-plugin/tree/main/docs)

The plugin is open source under Apache License 2.0 and actively maintained.
