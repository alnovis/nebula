---
title: "Proto Wrapper Plugin: How I Stopped Fighting Protobuf Versioning"
slug: "proto-wrapper-plugin"
description: "A tool born from frustration with multi-version protobuf schemas. How to handle protocol evolution without losing your mind."
date: "2025-01-25T10:00:00Z"
tags: ["java", "protobuf", "maven", "gradle", "code-generation"]
draft: false
---

If you've ever worked on a system that processes protobuf messages from multiple protocol versions, you know the pain. You add a new field, change a type from `int32` to `enum`, and suddenly your codebase is littered with `if (version == 1) ... else if (version == 2)` blocks.

I built Proto Wrapper Plugin to solve this problem once and for all.

## The Pain Point

Here's a typical scenario. You have a payment processing system. Version 1 of your protocol uses an integer for payment type:

```protobuf
// v1/payment.proto
message Payment {
  int32 payment_type = 1;  // 1=cash, 2=card, 3=transfer
}
```

Then someone decides (rightfully so) that enums are better:

```protobuf
// v2/payment.proto
message Payment {
  PaymentType payment_type = 1;
}

enum PaymentType {
  CASH = 1;
  CARD = 2;
  TRANSFER = 3;
}
```

Now your code looks like this:

```java
// This is everywhere in your codebase
if (protocolVersion == 1) {
    int type = paymentV1.getPaymentType();
    processPayment(type);
} else if (protocolVersion == 2) {
    PaymentType type = paymentV2.getPaymentType();
    processPayment(type.getNumber());
}
```

Multiply this by dozens of messages, and you have a maintenance nightmare.

## The Solution: One Interface to Rule Them All

Proto Wrapper generates a unified Java API that works across all your protocol versions:

```java
// Single code path. Works with v1, v2, v3... whatever.
Payment payment = versionContext.wrapPayment(anyProto);
int type = payment.getPaymentType();  // Always returns int
PaymentType typeEnum = payment.getPaymentTypeEnum();  // Returns enum when available

byte[] bytes = payment.toBytes();  // Serializes back to original version
```

The plugin analyzes your proto schemas, detects conflicts, and generates appropriate accessors. You write version-agnostic code, and the generated wrappers handle the details.

## Getting Started

Add the plugin to your Maven build:

```xml
<plugin>
    <groupId>io.alnovis</groupId>
    <artifactId>proto-wrapper-maven-plugin</artifactId>
    <version>2.2.0</version>
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

Or if you prefer Gradle:

```kotlin
plugins {
    id("io.alnovis.proto-wrapper") version "2.2.0"
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

Run `mvn generate-sources` or `./gradlew generateProtoWrapper`, and you get a clean API.

## What Conflicts Can It Handle?

Over time, I've added support for pretty much every type conflict I've encountered in production:

**INT_ENUM** — The most common one. Field starts as `int32`, becomes `enum`. You get both `getField()` (returns int) and `getFieldEnum()` (returns enum).

**ENUM_ENUM** — Two different enums across versions. Unified as int with enum conversion methods.

**WIDENING** — `int32` grows to `int64`. The wrapper uses `long` everywhere, with runtime validation for versions that need the narrower type.

**FLOAT_DOUBLE** — Precision change from `float` to `double`. Unified as `double`.

**SIGNED_UNSIGNED** — `int32` vs `uint32`, `sint32` vs `int32`. Unified as `long` with proper range validation.

**PRIMITIVE_MESSAGE** — A simple `int64 total` becomes `Money total` with amount and currency. You get `getTotal()` for primitive versions and `getTotalMessage()` for message versions.

**STRING_BYTES** — Someone decides `string` should be `bytes`. Dual accessors handle the conversion.

**REPEATED_SINGLE** — Field changes from singular to repeated or vice versa. Returns `List` in all cases.

**FIELD_RENUMBER** — Sometimes field numbers change between versions (legacy systems, don't ask). You can explicitly map them:

```xml
<fieldMappings>
    <fieldMapping>
        <message>Order</message>
        <fieldName>parent_ref</fieldName>
        <versionNumbers>
            <v1>3</v1>
            <v2>5</v2>
        </versionNumbers>
    </fieldMapping>
</fieldMappings>
```

The diff tool can even detect suspected renumbering and suggest the configuration.

## Batteries Included

A few things that make life easier:

**No protoc installation required.** The plugin downloads the right protoc binary for your platform automatically. Just works.

**Incremental builds.** Changed one proto file? Only affected wrappers regenerate. Saves 50%+ build time on large projects.

**Well-known types conversion.** `google.protobuf.Timestamp` becomes `java.time.Instant`. `Duration` becomes `java.time.Duration`. No more manual conversions.

**Builder pattern.** Full support for creating and modifying messages:

```java
Payment payment = Payment.newBuilder(ctx)
    .setPaymentType(PaymentType.CARD)
    .setAmount(Money.newBuilder(ctx)
        .setValue(1000)
        .setCurrency("USD")
        .build())
    .build();
```

**Spring Boot Starter.** For Spring Boot 3+ projects, there's an auto-configuration that handles version context per HTTP request.

## Schema Diff Tool

Before deploying a new protocol version, you probably want to know what changed. The built-in diff tool helps:

```bash
mvn proto-wrapper:diff -Dv1=proto/v1 -Dv2=proto/v2
```

It shows added/removed fields, type changes, and flags breaking changes. You can integrate it into CI to catch compatibility issues before they hit production:

```bash
mvn proto-wrapper:diff -Dv1=proto/production -Dv2=proto/development -DfailOnBreaking=true
```

Output formats include text, JSON, and Markdown for reports.

## Version Constants

No more magic strings. The plugin generates a `ProtocolVersions` class:

```java
// Compile-time constants
String version = ProtocolVersions.V1;

// Runtime validation
if (ProtocolVersions.isSupported(versionId)) {
    VersionContext ctx = VersionContext.forVersionId(versionId);
}

// Fail fast on unknown versions
ProtocolVersions.requireSupported(versionId);
```

## The Generated Code

Here's what the structure looks like:

```
com/example/model/
├── api/
│   ├── Payment.java           # Interface
│   ├── PaymentType.java       # Unified enum
│   ├── VersionContext.java    # Factory
│   ├── ProtocolVersions.java  # Constants
│   └── impl/
│       └── AbstractPayment.java
├── v1/
│   ├── PaymentV1.java
│   └── VersionContextV1.java
└── v2/
    ├── PaymentV2.java
    └── VersionContextV2.java
```

The generated code is plain Java. No runtime dependencies beyond protobuf itself. You can read it, debug it, understand it.

## Links

- [GitHub](https://github.com/alnovis/proto-wrapper-plugin)
- [Maven Central](https://central.sonatype.com/artifact/io.alnovis/proto-wrapper-core)
- [Full Documentation](https://github.com/alnovis/proto-wrapper-plugin/tree/main/docs)

The plugin is open source (Apache 2.0) and actively maintained. If you're dealing with protobuf versioning headaches, give it a try.
