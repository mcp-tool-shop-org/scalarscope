// Inference Optimization Fixture Tests
// Golden fixture tests for before/after inference optimization comparison.
// Uses xUnit with FluentAssertions.

using System.Linq;
using System.Reflection;
using System.Text.Json;
using FluentAssertions;
using ScalarScope.Services.Connectors;
using Xunit;

namespace ScalarScope.FixtureTests;

/// <summary>
/// Tests for the inference optimization golden fixtures.
/// Validates RunTrace loading, validation, comparison, and delta computation.
/// </summary>
public class InferenceOptimizationFixtureTests
{
    private readonly RunTraceValidator _validator = new();
    private readonly RunTraceComparer _comparer = new();

    #region A) Validator Tests - Positive Fixtures

    [Fact]
    public void Baseline_RunTrace_Should_Load_Successfully()
    {
        // Act
        var baseline = FixtureLoader.LoadRunTrace("baseline_tfrt_runtrace.json");

        // Assert
        baseline.Should().NotBeNull();
        baseline.RunId.Should().Be("tfrt-baseline-fixture-001");
        baseline.RunType.Should().Be(RunType.Inference);
        baseline.Framework.Should().Be(FrameworkType.TensorFlowRT);
        baseline.Label.Should().Be("Baseline");
    }

    [Fact]
    public void Optimized_RunTrace_Should_Load_Successfully()
    {
        // Act
        var optimized = FixtureLoader.LoadRunTrace("optimized_tfrt_runtrace.json");

        // Assert
        optimized.Should().NotBeNull();
        optimized.RunId.Should().Be("tfrt-optimized-fixture-001");
        optimized.RunType.Should().Be(RunType.Inference);
        optimized.Framework.Should().Be(FrameworkType.TensorFlowRT);
        optimized.Label.Should().Be("Optimized");
    }

    [Fact]
    public void Baseline_RunTrace_Should_Pass_Validation()
    {
        // Arrange
        var baseline = FixtureLoader.LoadRunTrace("baseline_tfrt_runtrace.json");

        // Act
        var result = _validator.Validate(baseline);

        // Assert
        result.IsValid.Should().BeTrue("baseline fixture should be valid");
        result.Errors.Should().BeEmpty("baseline fixture should have no errors");
    }

    [Fact]
    public void Optimized_RunTrace_Should_Pass_Validation()
    {
        // Arrange
        var optimized = FixtureLoader.LoadRunTrace("optimized_tfrt_runtrace.json");

        // Act
        var result = _validator.Validate(optimized);

        // Assert
        result.IsValid.Should().BeTrue("optimized fixture should be valid");
        result.Errors.Should().BeEmpty("optimized fixture should have no errors");
    }

    [Fact]
    public void Baseline_Should_Have_Strictly_Increasing_Steps()
    {
        // Arrange
        var baseline = FixtureLoader.LoadRunTrace("baseline_tfrt_runtrace.json");

        // Act
        var steps = baseline.Timeline.Steps.ToList();

        // Assert
        steps.Should().BeInAscendingOrder("steps must be strictly increasing");
        steps.Distinct().Should().HaveCount(steps.Count, "steps must not repeat");
    }

    [Fact]
    public void Baseline_Scalar_Lengths_Should_Match_Steps()
    {
        // Arrange
        var baseline = FixtureLoader.LoadRunTrace("baseline_tfrt_runtrace.json");
        var stepCount = baseline.Timeline.Steps.Count;

        // Act & Assert
        foreach (var series in baseline.Scalars.Series)
        {
            series.Values.Should().HaveCount(stepCount,
                $"scalar '{series.Name}' should have same length as steps");
        }
    }

    [Fact]
    public void Both_Fixtures_Should_Have_No_Trailing_Nulls_In_Latency()
    {
        // Arrange
        var baseline = FixtureLoader.LoadRunTrace("baseline_tfrt_runtrace.json");
        var optimized = FixtureLoader.LoadRunTrace("optimized_tfrt_runtrace.json");

        // Act
        var baselineLatency = baseline.Scalars.GetByName("latency_ms");
        var optimizedLatency = optimized.Scalars.GetByName("latency_ms");

        // Assert
        baselineLatency.Should().NotBeNull();
        optimizedLatency.Should().NotBeNull();
        baselineLatency!.Values.Last().Should().NotBeNull("baseline latency should have no trailing nulls");
        optimizedLatency!.Values.Last().Should().NotBeNull("optimized latency should have no trailing nulls");
    }

    #endregion

    #region B) Validator Tests - Negative Fixture

    [Fact]
    public void Broken_RunTrace_Should_Load_Successfully()
    {
        // Act - loading should succeed (it's valid JSON)
        var broken = FixtureLoader.LoadRunTrace("broken_baseline_tfrt_runtrace.json");

        // Assert
        broken.Should().NotBeNull();
        broken.RunId.Should().Be("tfrt-broken-baseline-fixture-001");
    }

    [Fact]
    public void Broken_RunTrace_Should_Fail_Validation_With_Stable_Error_Codes()
    {
        // Arrange
        var broken = FixtureLoader.LoadRunTrace("broken_baseline_tfrt_runtrace.json");

        // Act
        var result = _validator.Validate(broken);

        // Assert
        result.IsValid.Should().BeFalse("broken fixture should fail validation");
        result.Errors.Should().NotBeEmpty("broken fixture should have errors");
        
        // Order-independent assertion: check for expected error code(s)
        // Validator short-circuits on timeline error, so only RT_TIMELINE_NON_MONOTONIC is reported
        var errorCodes = result.Errors.Select(e => e.Code).ToHashSet();
        errorCodes.Should().Contain(RunTraceErrorCodes.RT_TIMELINE_NON_MONOTONIC,
            "should detect non-monotonic steps (order-independent check)");
    }

    [Fact]
    public void Broken_RunTrace_Timeline_Error_Should_Reference_Correct_Index()
    {
        // Arrange
        var broken = FixtureLoader.LoadRunTrace("broken_baseline_tfrt_runtrace.json");

        // Act
        var result = _validator.Validate(broken);

        // Assert
        var timelineError = result.Errors.FirstOrDefault(e => 
            e.Code == RunTraceErrorCodes.RT_TIMELINE_NON_MONOTONIC);
        timelineError.Should().NotBeNull();
        timelineError!.Message.Should().Contain("5", "error should reference index 5 where repeat occurs");
    }

    [Fact]
    public void Broken_RunTrace_Scalar_Error_Would_Reference_Latency_Series_If_Timeline_Valid()
    {
        // Note: Validator short-circuits on timeline errors, so scalar error won't appear
        // This test documents the expected behavior if scalar check was reached
        
        // Arrange
        var broken = FixtureLoader.LoadRunTrace("broken_baseline_tfrt_runtrace.json");

        // Act
        var result = _validator.Validate(broken);

        // Assert - timeline error prevents scalar check
        result.Errors.Should().Contain(e => e.Code == RunTraceErrorCodes.RT_TIMELINE_NON_MONOTONIC,
            "timeline error is detected first");
        
        // Scalar error is NOT present because validator short-circuits
        result.Errors.Should().NotContain(e => e.Code == RunTraceErrorCodes.RT_SCALAR_LENGTH_MISMATCH,
            "scalar check is skipped when timeline is invalid (by design)");
    }

    #endregion

    #region C) Comparison Tests

    [Fact]
    public void Comparison_Should_Use_TFRT_Preset()
    {
        // Arrange
        var baseline = FixtureLoader.LoadRunTrace("baseline_tfrt_runtrace.json");
        var optimized = FixtureLoader.LoadRunTrace("optimized_tfrt_runtrace.json");
        var intent = ComparisonIntent.TfrtOptimization();

        // Act
        var result = _comparer.Compare(baseline, optimized, intent);

        // Assert
        result.PresetId.Should().Be(TfrtRuntimePreset.PresetId);
        result.Intent.Alignment.Should().Be(AlignmentMode.RuntimeMilestone);
    }

    [Fact]
    public void Comparison_Should_Align_By_Steady_State_Start()
    {
        // Arrange
        var baseline = FixtureLoader.LoadRunTrace("baseline_tfrt_runtrace.json");
        var optimized = FixtureLoader.LoadRunTrace("optimized_tfrt_runtrace.json");
        var intent = ComparisonIntent.TfrtOptimization();

        // Act
        var result = _comparer.Compare(baseline, optimized, intent);

        // Assert
        result.Alignment.Mode.Should().Be(AlignmentMode.RuntimeMilestone);
        result.Alignment.AnchorType.Should().Be(RuntimeMilestoneType.SteadyStateStart);
        result.Alignment.AnchorStepA.Should().Be(13, "baseline steady_state_start is step 13");
        result.Alignment.AnchorStepB.Should().Be(7, "optimized steady_state_start is step 7");
    }

    [Fact]
    public void Comparison_Should_Show_DeltaTc_Present_And_Earlier_For_Optimized()
    {
        // Arrange
        var baseline = FixtureLoader.LoadRunTrace("baseline_tfrt_runtrace.json");
        var optimized = FixtureLoader.LoadRunTrace("optimized_tfrt_runtrace.json");
        var intent = ComparisonIntent.TfrtOptimization();

        // Act
        var result = _comparer.Compare(baseline, optimized, intent);

        // Assert
        var deltaTc = result.Deltas.FirstOrDefault(d => d.DeltaType == "ΔTc");
        deltaTc.Should().NotBeNull("ΔTc should be present");
        deltaTc!.Fired.Should().BeTrue("ΔTc should fire for this fixture pair");
        deltaTc.AbsoluteDifference.Should().BeLessThan(0, 
            "optimized should stabilize earlier (negative difference)");
        
        // Coarse assertion: at least 3 steps difference
        Math.Abs(deltaTc.AbsoluteDifference).Should().BeGreaterOrEqualTo(3,
            "ΔTc should show at least 3 steps difference");
    }

    [Fact]
    public void Comparison_Should_Suppress_DeltaTd()
    {
        // Arrange
        var baseline = FixtureLoader.LoadRunTrace("baseline_tfrt_runtrace.json");
        var optimized = FixtureLoader.LoadRunTrace("optimized_tfrt_runtrace.json");
        var intent = ComparisonIntent.TfrtOptimization();

        // Act
        var result = _comparer.Compare(baseline, optimized, intent);

        // Assert
        var deltaTd = result.Deltas.FirstOrDefault(d => d.DeltaType == "ΔTd");
        deltaTd.Should().NotBeNull("ΔTd should be present in results");
        deltaTd!.IsSuppressed.Should().BeTrue("ΔTd should be suppressed for TFRT runtime preset");
    }

    [Fact]
    public void Comparison_Should_Suppress_DeltaA()
    {
        // Arrange
        var baseline = FixtureLoader.LoadRunTrace("baseline_tfrt_runtrace.json");
        var optimized = FixtureLoader.LoadRunTrace("optimized_tfrt_runtrace.json");
        var intent = ComparisonIntent.TfrtOptimization();

        // Act
        var result = _comparer.Compare(baseline, optimized, intent);

        // Assert
        var deltaA = result.Deltas.FirstOrDefault(d => d.DeltaType == "ΔĀ");
        deltaA.Should().NotBeNull("ΔĀ should be present in results");
        deltaA!.IsSuppressed.Should().BeTrue("ΔĀ should be suppressed for TFRT runtime preset");
    }

    [Fact]
    public void Comparison_Should_Validate_Fingerprints()
    {
        // Arrange
        var baseline = FixtureLoader.LoadRunTrace("baseline_tfrt_runtrace.json");
        var optimized = FixtureLoader.LoadRunTrace("optimized_tfrt_runtrace.json");
        var intent = ComparisonIntent.TfrtOptimization();

        // Act
        var result = _comparer.Compare(baseline, optimized, intent);

        // Assert
        result.Fingerprints.IsValidForComparison.Should().BeTrue();
        
        // Dataset/code should match
        result.Fingerprints.Differences.Should().NotContain(d => 
            d.Category == "dataset" && !d.IsExpectedForOptimization,
            "dataset fingerprint mismatch should not exist");
        
        // Model may differ (expected for optimization)
        var modelDiff = result.Fingerprints.Differences.FirstOrDefault(d => d.Category == "model");
        if (modelDiff != null)
        {
            modelDiff.IsExpectedForOptimization.Should().BeTrue(
                "model fingerprint difference should be marked as expected");
        }
    }

    #endregion

    #region D) Review Bundle Export Tests

    [Fact]
    public void Review_Bundle_Should_Export_Successfully()
    {
        // Arrange
        var baseline = FixtureLoader.LoadRunTrace("baseline_tfrt_runtrace.json");
        var optimized = FixtureLoader.LoadRunTrace("optimized_tfrt_runtrace.json");
        var intent = ComparisonIntent.TfrtOptimization();
        var comparison = _comparer.Compare(baseline, optimized, intent);

        // Act
        var bundle = _comparer.ExportReviewBundle(comparison, baseline, optimized);

        // Assert
        bundle.Should().NotBeNull();
        bundle.BundleHash.Should().NotBeNullOrEmpty();
        bundle.Version.Should().Be("1.0.0");
        bundle.RecomputeDisabled.Should().BeTrue("review mode should disable recompute");
    }

    [Fact]
    public void Review_Bundle_Should_Include_Required_Fields()
    {
        // Arrange
        var baseline = FixtureLoader.LoadRunTrace("baseline_tfrt_runtrace.json");
        var optimized = FixtureLoader.LoadRunTrace("optimized_tfrt_runtrace.json");
        var intent = ComparisonIntent.TfrtOptimization();
        var comparison = _comparer.Compare(baseline, optimized, intent);

        // Act
        var bundle = _comparer.ExportReviewBundle(comparison, baseline, optimized);

        // Assert
        bundle.Comparison.DeltaSpecVersion.Should().NotBeNullOrEmpty();
        bundle.BundleHash.Should().NotBeNullOrEmpty();
        bundle.Comparison.ComparisonId.Should().NotBeNullOrEmpty();
        bundle.FingerprintSummary.Should().NotBeNull();
        bundle.Comparison.Alignment.Should().NotBeNull();
    }

    [Fact]
    public void Review_Bundle_Should_Preserve_Labels()
    {
        // Arrange
        var baseline = FixtureLoader.LoadRunTrace("baseline_tfrt_runtrace.json");
        var optimized = FixtureLoader.LoadRunTrace("optimized_tfrt_runtrace.json");
        var intent = ComparisonIntent.TfrtOptimization("Before XLA", "After XLA");
        var comparison = _comparer.Compare(baseline, optimized, intent);

        // Act
        var bundle = _comparer.ExportReviewBundle(comparison, baseline, optimized);

        // Assert
        bundle.FingerprintSummary.LabelA.Should().Be("Before XLA");
        bundle.FingerprintSummary.LabelB.Should().Be("After XLA");
    }

    [Fact]
    public void ReExport_Should_Stamp_Parent_Bundle_Hash()
    {
        // Arrange
        var baseline = FixtureLoader.LoadRunTrace("baseline_tfrt_runtrace.json");
        var optimized = FixtureLoader.LoadRunTrace("optimized_tfrt_runtrace.json");
        var intent = ComparisonIntent.TfrtOptimization();
        var comparison = _comparer.Compare(baseline, optimized, intent);
        var originalBundle = _comparer.ExportReviewBundle(comparison, baseline, optimized);

        // Act
        var reExported = _comparer.ReExportFromBundle(originalBundle, ReviewExportMode.Audit);

        // Assert
        reExported.IsReviewedFromBundle.Should().BeTrue();
        reExported.ParentBundleHash.Should().Be(originalBundle.BundleHash);
        reExported.ExportMode.Should().Be(ReviewExportMode.Audit);
    }

    #endregion

    #region E) Nearly Identical Fixture Tests (Suppression Stress)

    [Fact]
    public void NearlyIdentical_RunTraces_Should_Load_Successfully()
    {
        // Act
        var nearBaseline = FixtureLoader.LoadRunTrace("nearly_identical_baseline_tfrt_runtrace.json");
        var nearOptimized = FixtureLoader.LoadRunTrace("nearly_identical_optimized_tfrt_runtrace.json");

        // Assert
        nearBaseline.Should().NotBeNull();
        nearOptimized.Should().NotBeNull();
        nearBaseline.RunId.Should().Be("tfrt-near-baseline-fixture-001");
        nearOptimized.RunId.Should().Be("tfrt-near-optimized-fixture-001");
    }

    [Fact]
    public void NearlyIdentical_RunTraces_Should_Pass_Validation()
    {
        // Arrange
        var nearBaseline = FixtureLoader.LoadRunTrace("nearly_identical_baseline_tfrt_runtrace.json");
        var nearOptimized = FixtureLoader.LoadRunTrace("nearly_identical_optimized_tfrt_runtrace.json");

        // Act
        var resultA = _validator.Validate(nearBaseline);
        var resultB = _validator.Validate(nearOptimized);

        // Assert
        resultA.IsValid.Should().BeTrue();
        resultB.IsValid.Should().BeTrue();
    }

    [Fact]
    public void NearlyIdentical_Comparison_Should_Not_Fire_DeltaTc()
    {
        // Arrange
        var nearBaseline = FixtureLoader.LoadRunTrace("nearly_identical_baseline_tfrt_runtrace.json");
        var nearOptimized = FixtureLoader.LoadRunTrace("nearly_identical_optimized_tfrt_runtrace.json");
        var intent = ComparisonIntent.TfrtOptimization();

        // Act
        var result = _comparer.Compare(nearBaseline, nearOptimized, intent);

        // Assert
        var deltaTc = result.Deltas.FirstOrDefault(d => d.DeltaType == "ΔTc");
        deltaTc.Should().NotBeNull();
        
        // Either not fired, or fired with zero difference (same steady state step)
        if (deltaTc!.Fired)
        {
            deltaTc.AbsoluteDifference.Should().Be(0, 
                "nearly identical runs have same steady_state_start");
        }
    }

    [Fact]
    public void NearlyIdentical_Comparison_Should_Have_Zero_Fired_Deltas()
    {
        // Arrange
        var nearBaseline = FixtureLoader.LoadRunTrace("nearly_identical_baseline_tfrt_runtrace.json");
        var nearOptimized = FixtureLoader.LoadRunTrace("nearly_identical_optimized_tfrt_runtrace.json");
        var intent = ComparisonIntent.TfrtOptimization();

        // Act
        var result = _comparer.Compare(nearBaseline, nearOptimized, intent);

        // Assert - either no deltas fire, or they fire with effectively zero difference
        var meaningfulDeltas = result.FiredDeltas.Where(d => 
            !d.IsSuppressed && Math.Abs(d.AbsoluteDifference) > 0.001);
        
        meaningfulDeltas.Should().BeEmpty(
            "nearly identical runs should not produce meaningful deltas");
    }

    [Fact]
    public void NearlyIdentical_Should_Have_All_Preset_Suppressions_Active()
    {
        // Arrange
        var nearBaseline = FixtureLoader.LoadRunTrace("nearly_identical_baseline_tfrt_runtrace.json");
        var nearOptimized = FixtureLoader.LoadRunTrace("nearly_identical_optimized_tfrt_runtrace.json");
        var intent = ComparisonIntent.TfrtOptimization();

        // Act
        var result = _comparer.Compare(nearBaseline, nearOptimized, intent);

        // Assert
        var deltaTd = result.Deltas.FirstOrDefault(d => d.DeltaType == "ΔTd");
        var deltaA = result.Deltas.FirstOrDefault(d => d.DeltaType == "ΔĀ");
        
        deltaTd.Should().NotBeNull();
        deltaA.Should().NotBeNull();
        deltaTd!.IsSuppressed.Should().BeTrue("ΔTd should be suppressed");
        deltaA!.IsSuppressed.Should().BeTrue("ΔĀ should be suppressed");
    }

    [Fact]
    public void NearlyIdentical_Comparison_Should_Match_Expected_Assertions()
    {
        var nearBaseline = FixtureLoader.LoadRunTrace("nearly_identical_baseline_tfrt_runtrace.json");
        var nearOptimized = FixtureLoader.LoadRunTrace("nearly_identical_optimized_tfrt_runtrace.json");
        var assertions = FixtureLoader.LoadAssertions("expected_assertions_nearly_identical.json");
        var intent = ComparisonIntent.TfrtOptimization();

        var result = _comparer.Compare(nearBaseline, nearOptimized, intent);
        var expected = assertions.Expected!;

        ExpectDelta(result, "ΔTc", expected.ExpectedDeltas?.DeltaTc);
        ExpectDelta(result, "ΔO", expected.ExpectedDeltas?.DeltaO);
        ExpectDelta(result, "ΔF", expected.ExpectedDeltas?.DeltaF);
        ExpectDelta(result, "ΔTd", expected.ExpectedDeltas?.DeltaTd);
        ExpectDelta(result, "ΔĀ", expected.ExpectedDeltas?.DeltaA);

        expected.ExpectedOutcome.Should().NotBeNull();
        result.FiredDeltas.Count().Should().Be(expected.ExpectedOutcome!.DeltasFireCount);
        result.SuppressedDeltas.Count().Should().Be(expected.ExpectedOutcome.SuppressedCount);

        if (expected.Fingerprints?.ModelIdentical == true)
            nearBaseline.Metadata.ModelFingerprint.Should().Be(nearOptimized.Metadata.ModelFingerprint);
        if (expected.Fingerprints?.DatasetIdentical == true)
            nearBaseline.Metadata.DatasetFingerprint.Should().Be(nearOptimized.Metadata.DatasetFingerprint);
        if (expected.Fingerprints?.CodeIdentical == true)
            nearBaseline.Metadata.CodeFingerprint.Should().Be(nearOptimized.Metadata.CodeFingerprint);
        if (expected.Fingerprints?.EnvironmentIdentical == true)
            nearBaseline.Metadata.EnvironmentFingerprint.Should().Be(nearOptimized.Metadata.EnvironmentFingerprint);

        ExpectMilestone(nearBaseline, expected.Milestones?["baseline"]);
        ExpectMilestone(nearOptimized, expected.Milestones?["optimized"]);
    }

    #endregion

    #region F) Expected Assertions Validation

    [Fact]
    public void Expected_Assertions_Should_Load_Successfully()
    {
        // Act
        var assertions = FixtureLoader.LoadAssertions("expected_assertions.json");

        // Assert
        assertions.Should().NotBeNull();
        assertions.FixtureSet.Should().Be("InferenceOptimization_TFRT_v1");
        assertions.Expected.Should().NotBeNull();
    }

    [Fact]
    public void Comparison_Should_Match_Expected_Assertions()
    {
        // Arrange
        var baseline = FixtureLoader.LoadRunTrace("baseline_tfrt_runtrace.json");
        var optimized = FixtureLoader.LoadRunTrace("optimized_tfrt_runtrace.json");
        var assertions = FixtureLoader.LoadAssertions("expected_assertions.json");
        var intent = ComparisonIntent.TfrtOptimization();

        // Act
        var result = _comparer.Compare(baseline, optimized, intent);

        var expected = assertions.Expected!;
        var baselineValidation = _validator.Validate(baseline);
        var optimizedValidation = _validator.Validate(optimized);

        if (expected.Validation?.BaselineValid != null)
            baselineValidation.IsValid.Should().Be(expected.Validation.BaselineValid.Value);
        if (expected.Validation?.OptimizedValid != null)
            optimizedValidation.IsValid.Should().Be(expected.Validation.OptimizedValid.Value);
        if (expected.Validation?.NoErrors == true)
        {
            baselineValidation.Errors.Should().BeEmpty();
            optimizedValidation.Errors.Should().BeEmpty();
        }
        else if (expected.Validation?.NoErrors == false)
        {
            (baselineValidation.Errors.Count + optimizedValidation.Errors.Count).Should().BeGreaterThan(0);
        }
        if (expected.Validation?.BaselineRunId != null)
            baseline.RunId.Should().Be(expected.Validation.BaselineRunId);
        if (expected.Validation?.OptimizedRunId != null)
            optimized.RunId.Should().Be(expected.Validation.OptimizedRunId);

        ExpectCapabilities(baseline, expected.Capabilities?["baseline"]);
        ExpectCapabilities(optimized, expected.Capabilities?["optimized"]);
        ExpectMilestone(baseline, expected.Milestones?["baseline"]);
        ExpectMilestone(optimized, expected.Milestones?["optimized"]);
        ExpectFingerprints(baseline, optimized, expected.Fingerprints);

        if (expected.ComparisonIntent?.PresetId != null)
            result.PresetId.Should().Be(expected.ComparisonIntent.PresetId);
        if (expected.ComparisonIntent?.AlignmentMode != null)
            result.Intent.Alignment.ToString().Should().BeEquivalentTo(expected.ComparisonIntent.AlignmentMode);
        if (expected.ComparisonIntent?.PrimaryMilestone != null)
            MilestoneToken(result.Intent.PrimaryAnchor).Should().Be(expected.ComparisonIntent.PrimaryMilestone);
        if (expected.ComparisonIntent?.FallbackMilestone != null)
            MilestoneToken(result.Intent.FallbackAnchor).Should().Be(expected.ComparisonIntent.FallbackMilestone);
        if (expected.ComparisonIntent?.LabelA != null)
            result.Intent.LabelA.Should().Be(expected.ComparisonIntent.LabelA);
        if (expected.ComparisonIntent?.LabelB != null)
            result.Intent.LabelB.Should().Be(expected.ComparisonIntent.LabelB);

        ExpectDelta(result, "ΔTc", expected.ExpectedDeltas?.DeltaTc);
        ExpectDelta(result, "ΔO", expected.ExpectedDeltas?.DeltaO);
        ExpectDelta(result, "ΔF", expected.ExpectedDeltas?.DeltaF);
        ExpectDelta(result, "ΔTd", expected.ExpectedDeltas?.DeltaTd);
        ExpectDelta(result, "ΔĀ", expected.ExpectedDeltas?.DeltaA);

        ExpectSteadyState(baseline, optimized, expected.SteadyStateMetrics);
        ExpectBundle(result, baseline, optimized, expected.BundleExport);
    }

    [Fact]
    public void Broken_Fixture_Should_Match_Expected_Error_Assertions()
    {
        // Arrange
        var broken = FixtureLoader.LoadRunTrace("broken_baseline_tfrt_runtrace.json");
        var assertions = FixtureLoader.LoadAssertions("expected_assertions_broken.json");

        // Act
        var result = _validator.Validate(broken);

        // Assert
        var expected = assertions.ExpectedValidation!;
        expected.IsValid.Should().NotBeNull();
        expected.ErrorCount.Should().NotBeNull();
        result.IsValid.Should().Be(expected.IsValid!.Value);
        result.Errors.Should().HaveCount(expected.ErrorCount!.Value);
        result.Warnings.Should().HaveCount(expected.Warnings?.Count ?? 0);
        result.Infos.Should().HaveCount(expected.Infos?.Count ?? 0);

        expected.Errors.Should().NotBeNullOrEmpty();
        result.Errors.Select(e => e.Code).Should().Equal(expected.Errors!.Select(e => e.Code));

        for (var i = 0; i < expected.Errors.Count; i++)
        {
            var actual = result.Errors[i];
            var wanted = expected.Errors[i];
            actual.Severity.ToString().Should().Be(wanted.Severity);
            actual.Message.Should().Contain(wanted.MessageContains);
            actual.Path.Should().Contain(wanted.PathContains);
            wanted.Context.Should().NotBeNull();
            actual.Context.Should().NotBeNull();
            ReadInt(actual.Context!["index"]).Should().Be(ReadInt(wanted.Context!["indexOfViolation"]));
            ReadInt(actual.Context["previous"]).Should().Be(ReadInt(wanted.Context["repeatedValue"]));
            ReadInt(actual.Context["current"]).Should().Be(ReadInt(wanted.Context["repeatedValue"]));
        }

        assertions.TestCases.Should().NotBeNull();
        assertions.TestCases!.ValidatorRejectsTrace.Should().NotBeNull();
        assertions.TestCases.ValidatorRejectsTrace!.Input.Should().Be("broken_baseline_tfrt_runtrace.json");
        assertions.TestCases.ValidatorRejectsTrace.ExpectedResult.Should().Contain($"{result.Errors.Count} error");
        assertions.TestCases.ErrorCodesStable.Should().NotBeNull();
        assertions.TestCases.ErrorCodesStable!.AssertCodes.Should().Equal(result.Errors.Select(e => e.Code));
        assertions.TestCases.ErrorCodesStable.IndexOfViolation.Should().Be(ReadInt(expected.Errors[0].Context!["indexOfViolation"]));
        assertions.TestCases.ErrorCodesStable.RepeatedValue.Should().Be(ReadInt(expected.Errors[0].Context["repeatedValue"]));
        assertions.TestCases.ErrorCodesStable.Reason.Should().NotBeNullOrWhiteSpace();

        assertions.ExpectedBehavior.Should().NotBeNull();
        var behavior = assertions.ExpectedBehavior!;
        behavior.ComparisonBlocked.Should().BeTrue();
        behavior.DeltasComputed.Should().BeFalse();
        behavior.BundleExportDisabled.Should().BeTrue();
        behavior.ErrorExplanationShown.Should().BeTrue();
        behavior.PresetApplicationBlocked.Should().BeTrue();

        var comparison = _comparer.Compare(broken, broken, ComparisonIntent.TfrtOptimization());
        comparison.ComparisonBlocked.Should().BeTrue();
        comparison.Deltas.Should().BeEmpty();
        comparison.PresetId.Should().BeEmpty();
        comparison.Fingerprints.IsValidForComparison.Should().BeFalse();
        comparison.Alignment.AlignedStepCount.Should().Be(0);
        comparison.Alignment.IsSuccess.Should().BeFalse();

        var userMessage = assertions.ExpectedUserMessage;
        userMessage.Should().NotBeNull();
        comparison.UserMessage.Should().Contain(userMessage!.Title);
        comparison.UserMessage.Split(userMessage.Title).Length.Should().Be(2);
        foreach (var bullet in userMessage.Bullets ?? [])
            comparison.UserMessage.Should().Contain(bullet);
        foreach (var advice in userMessage.ActionableAdvice ?? [])
            comparison.UserMessage.Should().Contain(advice);

        var export = Assert.Throws<InvalidOperationException>(
            () => _comparer.ExportReviewBundle(comparison, broken, broken));
        export.Message.Should().Contain("Bundle export is disabled");
        export.Message.Should().Contain(comparison.UserMessage);
    }

    private static void ExpectCapabilities(RuntimeRunTrace trace, CapabilityExpectation? expected)
    {
        expected.Should().NotBeNull();
        if (expected!.HasLatency != null)
            trace.Capabilities.HasLatency.Should().Be(expected.HasLatency.Value);
        if (expected.HasThroughput != null)
            trace.Capabilities.HasThroughput.Should().Be(expected.HasThroughput.Value);
        if (expected.HasMemory != null)
            trace.Capabilities.HasMemory.Should().Be(expected.HasMemory.Value);
        if (expected.HasLoss != null)
            trace.Capabilities.HasLoss.Should().Be(expected.HasLoss.Value);
        if (expected.HasAccuracy != null)
            trace.Capabilities.HasAccuracy.Should().Be(expected.HasAccuracy.Value);
    }

    private static void ExpectMilestone(RuntimeRunTrace trace, MilestoneExpectation? expected)
    {
        expected.Should().NotBeNull();
        if (expected!.WarmupEnd != null)
            trace.Milestones.WarmupEndStep.Should().Be(expected.WarmupEnd);
        if (expected.SteadyStateStart != null)
            trace.Milestones.SteadyStateStartStep.Should().Be(expected.SteadyStateStart);
        if (expected.SteadyStateEnd != null)
            trace.Milestones.OfType(RuntimeMilestoneType.SteadyStateEnd).First().Step.Should().Be(expected.SteadyStateEnd);
    }

    private static void ExpectFingerprints(RuntimeRunTrace baseline, RuntimeRunTrace optimized, FingerprintsExpectation? expected)
    {
        expected.Should().NotBeNull();
        if (expected!.DatasetMustMatch == true)
            baseline.Metadata.DatasetFingerprint.Should().Be(optimized.Metadata.DatasetFingerprint);
        else if (expected.DatasetMustMatch == false)
            baseline.Metadata.DatasetFingerprint.Should().NotBe(optimized.Metadata.DatasetFingerprint);
        if (expected.CodeMustMatch == true)
            baseline.Metadata.CodeFingerprint.Should().Be(optimized.Metadata.CodeFingerprint);
        else if (expected.CodeMustMatch == false)
            baseline.Metadata.CodeFingerprint.Should().NotBe(optimized.Metadata.CodeFingerprint);
        if (expected.EnvironmentMustMatch == true)
            baseline.Metadata.EnvironmentFingerprint.Should().Be(optimized.Metadata.EnvironmentFingerprint);
        else if (expected.EnvironmentMustMatch == false)
            baseline.Metadata.EnvironmentFingerprint.Should().NotBe(optimized.Metadata.EnvironmentFingerprint);
        if (expected.ModelMayDiffer == false)
            baseline.Metadata.ModelFingerprint.Should().Be(optimized.Metadata.ModelFingerprint);

        expected.Baseline.Should().NotBeNull();
        expected.Optimized.Should().NotBeNull();
        baseline.Metadata.ModelFingerprint.Should().Be(expected.Baseline!.Model);
        baseline.Metadata.DatasetFingerprint.Should().Be(expected.Baseline.Dataset);
        baseline.Metadata.CodeFingerprint.Should().Be(expected.Baseline.Code);
        baseline.Metadata.EnvironmentFingerprint.Should().Be(expected.Baseline.Environment);
        optimized.Metadata.ModelFingerprint.Should().Be(expected.Optimized!.Model);
        optimized.Metadata.DatasetFingerprint.Should().Be(expected.Optimized.Dataset);
        optimized.Metadata.CodeFingerprint.Should().Be(expected.Optimized.Code);
        optimized.Metadata.EnvironmentFingerprint.Should().Be(expected.Optimized.Environment);
    }

    private static void ExpectDelta(ComparisonResult result, string deltaType, DeltaExpectation? expected)
    {
        if (expected == null)
            return;

        var delta = result.Deltas.FirstOrDefault(d => d.DeltaType == deltaType);
        if (expected.ShouldBePresent == true)
            delta.Should().NotBeNull($"{deltaType} is recorded as present");
        else if (expected.ShouldBePresent == false)
            delta.Should().BeNull($"{deltaType} is recorded as absent");

        if (expected.Fired != null)
        {
            delta.Should().NotBeNull();
            delta!.Fired.Should().Be(expected.Fired.Value, $"{deltaType} fired flag");
        }
        if (expected.ShouldBeSuppressed != null)
        {
            delta.Should().NotBeNull();
            delta!.IsSuppressed.Should().Be(expected.ShouldBeSuppressed.Value, $"{deltaType} suppression");
        }
        if (expected.MinDeltaSteps != null)
        {
            delta.Should().NotBeNull();
            Math.Abs(delta!.AbsoluteDifference).Should().BeGreaterThanOrEqualTo(expected.MinDeltaSteps.Value);
        }
        if (expected.Direction == "optimized_stabilizes_earlier")
        {
            delta.Should().NotBeNull();
            delta!.ValueB.Should().BeLessThan(delta.ValueA);
        }
    }

    private static void ExpectSteadyState(RuntimeRunTrace baseline, RuntimeRunTrace optimized, SteadyStateMetricsExpectation? expected)
    {
        expected.Should().NotBeNull();
        expected!.Baseline.Should().NotBeNull();
        expected.Optimized.Should().NotBeNull();
        expected.Improvement.Should().NotBeNull();

        var baseStart = baseline.Milestones.SteadyStateStartStep!.Value;
        var baseEnd = baseline.Milestones.OfType(RuntimeMilestoneType.SteadyStateEnd).First().Step;
        var optStart = optimized.Milestones.SteadyStateStartStep!.Value;
        var optEnd = optimized.Milestones.OfType(RuntimeMilestoneType.SteadyStateEnd).First().Step;

        var (baseLatency, baseLatencyStd) = Steady(baseline, "latency_ms", baseStart, baseEnd);
        var (optLatency, optLatencyStd) = Steady(optimized, "latency_ms", optStart, optEnd);
        var (baseThroughput, _) = Steady(baseline, "throughput_items_per_sec", baseStart, baseEnd);
        var (optThroughput, _) = Steady(optimized, "throughput_items_per_sec", optStart, optEnd);

        baseLatency.Should().BeApproximately(expected.Baseline!.LatencyMean!.Value, 0.05);
        baseLatencyStd.Should().BeApproximately(expected.Baseline.LatencyStdDev!.Value, 0.05);
        baseThroughput.Should().BeApproximately(expected.Baseline.ThroughputMean!.Value, 0.05);
        optLatency.Should().BeApproximately(expected.Optimized!.LatencyMean!.Value, 0.05);
        optLatencyStd.Should().BeApproximately(expected.Optimized.LatencyStdDev!.Value, 0.05);
        optThroughput.Should().BeApproximately(expected.Optimized.ThroughputMean!.Value, 0.05);

        $"{Math.Round((baseLatency - optLatency) / baseLatency * 100.0):0}%"
            .Should().Be(expected.Improvement!.LatencyReduction);
        $"{Math.Round((optThroughput - baseThroughput) / baseThroughput * 100.0):0}%"
            .Should().Be(expected.Improvement.ThroughputIncrease);
    }

    private void ExpectBundle(ComparisonResult result, RuntimeRunTrace baseline, RuntimeRunTrace optimized, BundleExportExpectation? expected)
    {
        expected.Should().NotBeNull();
        expected!.MustInclude.Should().NotBeNullOrEmpty();

        ReviewOnlyBundle? bundle = null;
        var exportThrew = false;
        try
        {
            bundle = _comparer.ExportReviewBundle(result, baseline, optimized);
        }
        catch
        {
            exportThrew = true;
        }

        if (expected.ShouldSucceed == true)
        {
            exportThrew.Should().BeFalse();
            bundle.Should().NotBeNull();
        }
        else if (expected.ShouldSucceed == false)
        {
            (exportThrew || bundle == null).Should().BeTrue();
            return;
        }

        var members = typeof(ReviewOnlyBundle).GetProperties(BindingFlags.Instance | BindingFlags.Public)
            .Select(property => property.Name)
            .ToHashSet(StringComparer.Ordinal);
        foreach (var name in expected.MustInclude!)
        {
            members.Should().Contain(name, $"{name} is not a ReviewOnlyBundle member");
            var value = typeof(ReviewOnlyBundle).GetProperty(name)!.GetValue(bundle);
            value.Should().NotBeNull($"{name} was missing on the exported bundle");
        }

        if (expected.ReviewModeEnabled == true)
            bundle!.ExportMode.Should().Be(ReviewExportMode.Review);
        else if (expected.ReviewModeEnabled == false)
            bundle!.ExportMode.Should().NotBe(ReviewExportMode.Review);
        if (expected.RecomputeDisabled != null)
            bundle!.RecomputeDisabled.Should().Be(expected.RecomputeDisabled.Value);
    }

    private static (double Mean, double StdDev) Steady(RuntimeRunTrace trace, string seriesName, int start, int end)
    {
        var series = trace.Scalars.GetByName(seriesName);
        series.Should().NotBeNull();
        var values = new List<double>();
        for (var i = 0; i < trace.Timeline.Steps.Count; i++)
        {
            var step = trace.Timeline.Steps[i];
            if (step >= start && step <= end && series!.Values[i].HasValue)
                values.Add(series.Values[i]!.Value);
        }

        values.Should().NotBeEmpty();
        var mean = values.Average();
        var variance = values.Sum(value => (value - mean) * (value - mean)) / values.Count;
        return (mean, Math.Sqrt(variance));
    }

    private static string? MilestoneToken(RuntimeMilestoneType? type) => type switch
    {
        RuntimeMilestoneType.SteadyStateStart => "steady_state_start",
        RuntimeMilestoneType.SteadyStateEnd => "steady_state_end",
        RuntimeMilestoneType.WarmupEnd => "warmup_end",
        null => null,
        _ => type.ToString()
    };

    private static int ReadInt(object? value) => value switch
    {
        JsonElement element when element.ValueKind == JsonValueKind.Number => element.GetInt32(),
        JsonElement element when element.ValueKind == JsonValueKind.String => int.Parse(element.GetString()!),
        int number => number,
        long number => (int)number,
        double number => (int)number,
        _ => throw new InvalidOperationException($"Cannot read an integer from '{value}'")
    };

    #endregion

    #region G) Capability Tests

    [Fact]
    public void Baseline_Should_Have_Expected_Capabilities()
    {
        // Arrange
        var baseline = FixtureLoader.LoadRunTrace("baseline_tfrt_runtrace.json");

        // Assert
        baseline.Capabilities.HasLatency.Should().BeTrue("baseline has latency_ms");
        baseline.Capabilities.HasThroughput.Should().BeTrue("baseline has throughput");
        baseline.Capabilities.HasMemory.Should().BeTrue("baseline has memory_mb");
        baseline.Capabilities.HasLoss.Should().BeFalse("inference doesn't have loss");
        baseline.Capabilities.HasAccuracy.Should().BeFalse("inference doesn't have accuracy");
    }

    [Fact]
    public void Optimized_Should_Have_Expected_Capabilities()
    {
        // Arrange
        var optimized = FixtureLoader.LoadRunTrace("optimized_tfrt_runtrace.json");

        // Assert
        optimized.Capabilities.HasLatency.Should().BeTrue("optimized has latency_ms");
        optimized.Capabilities.HasThroughput.Should().BeTrue("optimized has throughput");
        optimized.Capabilities.HasMemory.Should().BeTrue("optimized has memory_mb");
        optimized.Capabilities.HasProfiler.Should().BeTrue("optimized has profiler artifact");
    }

    #endregion

    #region H) Milestone Tests

    [Fact]
    public void Baseline_Should_Have_Expected_Milestones()
    {
        // Arrange
        var baseline = FixtureLoader.LoadRunTrace("baseline_tfrt_runtrace.json");

        // Assert
        baseline.Milestones.WarmupEndStep.Should().Be(12);
        baseline.Milestones.SteadyStateStartStep.Should().Be(13);
    }

    [Fact]
    public void Optimized_Should_Have_Expected_Milestones()
    {
        // Arrange
        var optimized = FixtureLoader.LoadRunTrace("optimized_tfrt_runtrace.json");

        // Assert
        optimized.Milestones.WarmupEndStep.Should().Be(6);
        optimized.Milestones.SteadyStateStartStep.Should().Be(7);
    }

    [Fact]
    public void Optimized_Should_Stabilize_Earlier_Than_Baseline()
    {
        // Arrange
        var baseline = FixtureLoader.LoadRunTrace("baseline_tfrt_runtrace.json");
        var optimized = FixtureLoader.LoadRunTrace("optimized_tfrt_runtrace.json");

        // Assert
        optimized.Milestones.SteadyStateStartStep.Should().NotBeNull();
        baseline.Milestones.SteadyStateStartStep.Should().NotBeNull();
        optimized.Milestones.SteadyStateStartStep!.Value.Should().BeLessThan(
            baseline.Milestones.SteadyStateStartStep!.Value,
            "optimized should reach steady state earlier");
    }

    #endregion
}
