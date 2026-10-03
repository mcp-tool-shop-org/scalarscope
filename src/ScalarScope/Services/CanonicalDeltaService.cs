using ScalarScope.Models;

namespace ScalarScope.Services;

/// <summary>
/// Computes the ≤5 canonical deltas that matter for comparison.
/// Each delta has a clear name, one-line explanation, and visual reference.
/// Phase 3: Make Comparison the Star
/// 
/// Rule: No delta exists unless it (1) changes interpretation and (2) can be visually grounded.
/// 
/// Implementation follows the TypeScript spec for cross-platform consistency.
/// </summary>
public static class CanonicalDeltaService
{
    /// <summary>Default detector configuration.</summary>
    public static readonly DeltaDetectorConfig DefaultConfig = new();

    /// <summary>
    /// Compute full delta result with alignment map.
    /// </summary>
    public static DeltaComputationResult ComputeDeltasWithAlignment(
        GeometryRun? leftRun,
        GeometryRun? rightRun,
        TemporalAlignment alignment,
        double currentTime,
        DeltaDetectorConfig? config = null)
    {
        config ??= DefaultConfig;
        
        var alignmentMap = AlignmentMapper.CreateAlignmentMap(leftRun, rightRun, alignment);
        var deltas = ComputeDeltas(leftRun, rightRun, alignment, currentTime, config, alignmentMap);
        var summary = GenerateAutoSummary(deltas);

        // Fingerprints live on this result. A null run must not reuse the previous call.
        var leftFingerprint = DeterminismService.HashRun(leftRun);
        var rightFingerprint = DeterminismService.HashRun(rightRun);
        var inputFingerprint = DeterminismService.CombineRunFingerprints(leftFingerprint, rightFingerprint);
        var deltaHash = DeterminismService.ComputeDeltaHash(deltas);
        var reproducibility = DeterminismService.GetReproducibilityMetadata(inputFingerprint);

        return new DeltaComputationResult
        {
            Alignment = alignmentMap,
            Deltas = deltas,
            ComparativeSummary = summary,
            DeltaHash = deltaHash,
            InputFingerprint = inputFingerprint,
            LeftRunFingerprint = leftFingerprint,
            RightRunFingerprint = rightFingerprint,
            Reproducibility = reproducibility
        };
    }

    /// <summary>
    /// Compute all canonical deltas between two runs at the current alignment.
    /// Returns deltas ordered by causal salience (not mathematical complexity):
    /// 1. Failure/Collapse → 2. Convergence → 3. Structural Emergence → 4. Evaluator Alignment → 5. Stability
    /// </summary>
    public static List<CanonicalDelta> ComputeDeltas(
        GeometryRun? leftRun,
        GeometryRun? rightRun,
        TemporalAlignment alignment,
        double currentTime,
        DeltaDetectorConfig? config = null,
        AlignmentMap? alignmentMap = null)
    {
        var deltas = new List<CanonicalDelta>();
        
        if (leftRun == null || rightRun == null)
            return deltas;

        config ??= DefaultConfig;
        alignmentMap ??= AlignmentMapper.CreateAlignmentMap(leftRun, rightRun, alignment);

        // Get total steps for conversions
        var leftSteps = leftRun.Trajectory?.Timesteps?.Count ?? 100;
        var rightSteps = rightRun.Trajectory?.Timesteps?.Count ?? 100;

        // Order by causal salience (most impactful first)
        
        // Δ1. Failure / Collapse Presence (highest priority - did something break?)
        var failureDelta = DetectFailurePresence(leftRun, rightRun, config.Failure, leftSteps, rightSteps);
        if (failureDelta.Status != DeltaStatus.Suppressed)
            deltas.Add(failureDelta);

        // Δ2. Convergence Timing Delta (did one run settle earlier?)
        var convergenceDelta = DetectConvergenceTiming(leftRun, rightRun, config.Convergence, leftSteps, rightSteps);
        if (convergenceDelta.Status != DeltaStatus.Suppressed)
            deltas.Add(convergenceDelta);

        // Δ3. Structural Emergence (did structure emerge differently?)
        var emergenceDelta = DetectStructuralEmergence(leftRun, rightRun, config.Emergence);
        if (emergenceDelta.Status != DeltaStatus.Suppressed)
            deltas.Add(emergenceDelta);

        // Δ4. Evaluator Alignment Divergence (did evaluators agree differently?)
        var evalAlignmentDelta = DetectEvaluatorAlignment(leftRun, rightRun, config.Alignment, currentTime, alignmentMap);
        if (evalAlignmentDelta.Status != DeltaStatus.Suppressed)
            deltas.Add(evalAlignmentDelta);

        // Δ5. Stability / Oscillation Difference (did one run wobble more?)
        var stabilityDelta = DetectStabilityOscillation(leftRun, rightRun, config.Stability, currentTime, alignmentMap);
        if (stabilityDelta.Status != DeltaStatus.Suppressed)
            deltas.Add(stabilityDelta);

        return deltas;
    }

    // ========================================================================
    // DETECTOR METHODS - TypeScript Spec Implementation
    // ========================================================================

    /// <summary>
    /// Δ1. Failure / Collapse Presence - "Did something break?"
    /// </summary>
    private static CanonicalDelta DetectFailurePresence(
        GeometryRun leftRun,
        GeometryRun rightRun,
        FailureConfig config,
        int leftSteps,
        int rightSteps)
    {
        var leftHasFailure = HasPersistentFailure(leftRun, out var leftFailureTime, out var leftFailureType);
        var rightHasFailure = HasPersistentFailure(rightRun, out var rightFailureTime, out var rightFailureType);

        int leftFailStep = (int)(leftFailureTime * leftSteps);
        int rightFailStep = (int)(rightFailureTime * rightSteps);

        // Suppression: neither run fails
        if (!leftHasFailure && !rightHasFailure)
        {
            return new CanonicalDelta
            {
                Id = DeltaIds.FailurePresence,
                Name = "Failure Events",
                Explanation = "No failure detected",
                Status = DeltaStatus.Suppressed,
                Notes = ["Neither run experienced failure events"],
                FailedA = false,
                FailedB = false
            };
        }

        string explanation;
        double visualAnchorTime;

        if (leftHasFailure && rightHasFailure)
        {
            if (Math.Abs(leftFailureTime - rightFailureTime) <= 1e-12)
            {
                explanation = "Both paths experienced instability; the failures were simultaneous";
            }
            else
            {
                var earlier = leftFailureTime < rightFailureTime ? "Path A" : "Path B";
                explanation = $"Both paths experienced instability; {earlier} first";
            }
            visualAnchorTime = Math.Min(leftFailureTime, rightFailureTime);
        }
        else if (leftHasFailure)
        {
            explanation = $"Only Path A experienced {leftFailureType} near step {leftFailStep}";
            visualAnchorTime = leftFailureTime;
        }
        else
        {
            explanation = $"Only Path B experienced {rightFailureType} near step {rightFailStep}";
            visualAnchorTime = rightFailureTime;
        }

        var anchors = new List<VisualAnchor>();
        if (leftHasFailure || rightHasFailure)
        {
            anchors.Add(new VisualAnchor
            {
                TargetView = "loss",
                RangeA = leftHasFailure ? (leftFailStep, leftFailStep + 10) : null,
                RangeB = rightHasFailure ? (rightFailStep, rightFailStep + 10) : null
            });
        }

        return new CanonicalDelta
        {
            Id = DeltaIds.FailurePresence,
            Name = "Failure Events",
            Explanation = explanation,
            Status = DeltaStatus.Present,
            LeftValue = leftHasFailure ? 1 : 0,
            RightValue = rightHasFailure ? 1 : 0,
            Delta = (rightHasFailure ? 1 : 0) - (leftHasFailure ? 1 : 0),
            Magnitude = 1.0,
            VisualAnchorTime = visualAnchorTime,
            DeltaType = DeltaType.Event,
            Anchors = anchors,
            FailedA = leftHasFailure,
            FailedB = rightHasFailure,
            TFailA = leftHasFailure ? leftFailStep : null,
            TFailB = rightHasFailure ? rightFailStep : null,
            FailureKindA = leftHasFailure ? leftFailureType : null,
            FailureKindB = rightHasFailure ? rightFailureType : null
        };
    }

    /// <summary>
    /// Δ2. Convergence Timing Delta - "Did one run settle earlier?"
    /// Phase 3.2: Step-based resolution is primary; normalized is for display only.
    /// </summary>
    private static CanonicalDelta DetectConvergenceTiming(
        GeometryRun leftRun,
        GeometryRun rightRun,
        ConvergenceConfig config,
        int leftSteps,
        int rightSteps)
    {
        var (leftConvergenceStep, leftConvergenceTime, leftEpsilon, leftTailLength, leftViolations) = 
            EstimateConvergenceTimeWithConfidence(leftRun, config);
        var (rightConvergenceStep, rightConvergenceTime, rightEpsilon, rightTailLength, rightViolations) = 
            EstimateConvergenceTimeWithConfidence(rightRun, config);

        // Suppression: Neither run converges
        if (leftConvergenceStep < 0 && rightConvergenceStep < 0)
        {
            return new CanonicalDelta
            {
                Id = DeltaIds.ConvergenceTiming,
                Name = "Convergence",
                Explanation = "No convergence detected",
                Status = DeltaStatus.Suppressed,
                Notes = ["Neither run converged within observation window"],
                TcA = null,
                TcB = null,
                DeltaTcSteps = null,
                DeltaTcNormalized = null,
                ConvergenceConfidence = null,
                WindowUsed = config.Window
            };
        }

        // Helper to compute normalized delta and confidence
        int compareLength = Math.Max(1, Math.Min(leftSteps, rightSteps));
        
        double ComputeConfidence(int tailLength, int violations, double epsilon, int totalSteps) 
        {
            if (totalSteps <= 0) return 0.5;
            // Factors: tail length, violation count, noise level
            double tailScore = Math.Min(1.0, tailLength / (double)Math.Max(config.Window * 2, 10));
            double violationScore = Math.Max(0.0, 1.0 - violations / (double)Math.Max(config.Window, 3));
            double noiseScore = Math.Max(0.0, 1.0 - Math.Min(epsilon / 0.1, 1.0)); // Lower epsilon = higher confidence
            return (tailScore * 0.4 + violationScore * 0.4 + noiseScore * 0.2);
        }

        // Handle indeterminate cases (one converged, one didn't) - Phase 3.2: PRESENT not suppressed
        if (leftConvergenceStep < 0 || rightConvergenceStep < 0)
        {
            var converged = leftConvergenceStep >= 0 ? "Path A" : "Path B";
            var notConverged = leftConvergenceStep >= 0 ? "Path B" : "Path A";
            var convergenceStep = Math.Max(leftConvergenceStep, rightConvergenceStep);
            var tailLength = leftConvergenceStep >= 0 ? leftTailLength : rightTailLength;
            var violations = leftConvergenceStep >= 0 ? leftViolations : rightViolations;
            var usedEpsilon = leftConvergenceStep >= 0 ? leftEpsilon : rightEpsilon;
            var totalSteps = leftConvergenceStep >= 0 ? leftSteps : rightSteps;
            var confidence = ComputeConfidence(tailLength, violations, usedEpsilon, totalSteps);
            
            return new CanonicalDelta
            {
                Id = DeltaIds.ConvergenceTiming,
                Name = "Convergence",
                Explanation = $"{converged} stabilized; {notConverged} did not within observed steps",
                Status = DeltaStatus.Present,
                LeftValue = leftConvergenceTime,
                RightValue = rightConvergenceTime,
                Delta = 1.0,
                Magnitude = 1.0,
                Confidence = confidence,
                VisualAnchorTime = leftConvergenceStep >= 0 ? leftConvergenceTime : rightConvergenceTime,
                DeltaType = DeltaType.Timing,
                Anchors = [new VisualAnchor
                {
                    TargetView = "loss",
                    RangeA = leftConvergenceStep >= 0 ? (leftConvergenceStep, leftConvergenceStep + config.Window) : (leftSteps - config.Window, leftSteps),
                    RangeB = rightConvergenceStep >= 0 ? (rightConvergenceStep, rightConvergenceStep + config.Window) : (rightSteps - config.Window, rightSteps)
                }],
                TcA = leftConvergenceStep >= 0 ? leftConvergenceStep : null,
                TcB = rightConvergenceStep >= 0 ? rightConvergenceStep : null,
                DeltaTcSteps = null, // Not applicable when only one converges
                DeltaTcNormalized = null,
                EpsilonUsed = usedEpsilon,
                ConvergenceConfidence = confidence,
                WindowUsed = config.Window
            };
        }

        var stepDiff = rightConvergenceStep - leftConvergenceStep;
        var absStepDiff = Math.Abs(stepDiff);
        var normalizedDelta = stepDiff / (double)compareLength;

        // Phase 3.2: Suppression uses ONLY ResolutionSteps (not normalized)
        if (absStepDiff < config.ResolutionSteps)
        {
            return new CanonicalDelta
            {
                Id = DeltaIds.ConvergenceTiming,
                Name = "Convergence",
                Explanation = "Similar convergence timing",
                Status = DeltaStatus.Suppressed,
                LeftValue = leftConvergenceTime,
                RightValue = rightConvergenceTime,
                Delta = rightConvergenceTime - leftConvergenceTime,
                Magnitude = Math.Abs(rightConvergenceTime - leftConvergenceTime),
                Notes = [$"Both runs converged near step {leftConvergenceStep} (Δ={absStepDiff} < {config.ResolutionSteps} steps)"],
                TcA = leftConvergenceStep,
                TcB = rightConvergenceStep,
                DeltaTcSteps = stepDiff,
                DeltaTcNormalized = normalizedDelta,
                WindowUsed = config.Window
            };
        }
        
        // Compute combined confidence
        var leftConfidence = ComputeConfidence(leftTailLength, leftViolations, leftEpsilon, leftSteps);
        var rightConfidence = ComputeConfidence(rightTailLength, rightViolations, rightEpsilon, rightSteps);
        var combinedConfidence = Math.Min(leftConfidence, rightConfidence);
        
        var faster = stepDiff > 0 ? "Path A" : "Path B";
        var explanation = $"{faster} converged {absStepDiff} steps earlier";

        return new CanonicalDelta
        {
            Id = DeltaIds.ConvergenceTiming,
            Name = "Convergence",
            Explanation = explanation,
            SummarySentence = $"{faster} settled {absStepDiff} steps before the other path",
            Status = DeltaStatus.Present,
            LeftValue = leftConvergenceTime,
            RightValue = rightConvergenceTime,
            Delta = rightConvergenceTime - leftConvergenceTime,
            Magnitude = Math.Abs(rightConvergenceTime - leftConvergenceTime),
            Confidence = combinedConfidence,
            VisualAnchorTime = Math.Min(leftConvergenceTime, rightConvergenceTime),
            DeltaType = DeltaType.Timing,
            Anchors = [new VisualAnchor
            {
                TargetView = "loss",
                RangeA = (leftConvergenceStep, leftConvergenceStep + config.Window),
                RangeB = (rightConvergenceStep, rightConvergenceStep + config.Window)
            }],
            TcA = leftConvergenceStep,
            TcB = rightConvergenceStep,
            DeltaTcSteps = stepDiff,
            DeltaTcNormalized = normalizedDelta,
            EpsilonUsed = leftEpsilon,
            ConvergenceConfidence = combinedConfidence,
            WindowUsed = config.Window
        };
    }

    /// <summary>
    /// Δ3. Structural Emergence - "Did structure emerge differently?"
    /// Phase 3.2: Uses recurrence rule to detect flicker-before-stabilize patterns.
    /// </summary>
    private static CanonicalDelta DetectStructuralEmergence(
        GeometryRun leftRun,
        GeometryRun rightRun,
        EmergenceConfig config)
    {
        var (leftDominanceStep, leftDominanceTime, leftTrigger) = FindDominanceOnsetTimeWithRecurrence(leftRun, config);
        var (rightDominanceStep, rightDominanceTime, rightTrigger) = FindDominanceOnsetTimeWithRecurrence(rightRun, config);

        // Suppression: dominance never achieved in either run
        if (leftDominanceStep < 0 && rightDominanceStep < 0)
        {
            return new CanonicalDelta
            {
                Id = DeltaIds.StructuralEmergence,
                Name = "Structure",
                Explanation = "No structural dominance",
                Status = DeltaStatus.Suppressed,
                Notes = ["Neither run achieved eigenvalue dominance"],
                TdA = null,
                TdB = null,
                DominanceRatioK = config.K
            };
        }

        // Helper to compute anchor duration based on trigger type
        int GetAnchorDuration(string? trigger) => trigger == "recurrence" ? config.RecurrenceWindow : config.Window;

        // Handle case where only one achieves dominance
        if (leftDominanceStep < 0)
        {
            var anchorDuration = GetAnchorDuration(rightTrigger);
            var triggerNote = rightTrigger == "recurrence" ? " (detected via recurrence)" : "";
            return new CanonicalDelta
            {
                Id = DeltaIds.StructuralEmergence,
                Name = "Structure",
                Explanation = $"Path B developed dominant direction; Path A remained distributed{triggerNote}",
                Status = DeltaStatus.Present,
                LeftValue = -1,
                RightValue = rightDominanceTime,
                Delta = 1.0,
                Magnitude = 1.0,
                VisualAnchorTime = rightDominanceTime,
                DeltaType = DeltaType.Timing,
                Anchors = [new VisualAnchor
                {
                    TargetView = "eigenvalues",
                    RangeB = (rightDominanceStep, rightDominanceStep + anchorDuration)
                }],
                TdA = null,
                TdB = rightDominanceStep,
                DominanceRatioK = config.K
            };
        }

        if (rightDominanceStep < 0)
        {
            var anchorDuration = GetAnchorDuration(leftTrigger);
            var triggerNote = leftTrigger == "recurrence" ? " (detected via recurrence)" : "";
            return new CanonicalDelta
            {
                Id = DeltaIds.StructuralEmergence,
                Name = "Structure",
                Explanation = $"Path A developed dominant direction; Path B remained distributed{triggerNote}",
                Status = DeltaStatus.Present,
                LeftValue = leftDominanceTime,
                RightValue = -1,
                Delta = -1.0,
                Magnitude = 1.0,
                VisualAnchorTime = leftDominanceTime,
                DeltaType = DeltaType.Timing,
                Anchors = [new VisualAnchor
                {
                    TargetView = "eigenvalues",
                    RangeA = (leftDominanceStep, leftDominanceStep + anchorDuration)
                }],
                TdA = leftDominanceStep,
                TdB = null,
                DominanceRatioK = config.K
            };
        }

        var stepDiff = Math.Abs(rightDominanceStep - leftDominanceStep);

        // Suppression: both runs achieve dominance simultaneously
        if (stepDiff < config.ResolutionSteps)
        {
            return new CanonicalDelta
            {
                Id = DeltaIds.StructuralEmergence,
                Name = "Structure",
                Explanation = "Similar emergence timing",
                Status = DeltaStatus.Suppressed,
                LeftValue = leftDominanceTime,
                RightValue = rightDominanceTime,
                Delta = rightDominanceTime - leftDominanceTime,
                Magnitude = Math.Abs(rightDominanceTime - leftDominanceTime),
                Notes = [$"Both runs achieved dominance near step {leftDominanceStep}"],
                TdA = leftDominanceStep,
                TdB = rightDominanceStep,
                DominanceRatioK = config.K
            };
        }

        var earlier = rightDominanceStep > leftDominanceStep ? "Path A" : "Path B";
        var earlierTrigger = rightDominanceStep > leftDominanceStep ? leftTrigger : rightTrigger;
        var triggerSuffix = earlierTrigger == "recurrence" ? " (structure forming)" : "";
        var explanation = $"{earlier} developed dominant direction {stepDiff} steps earlier{triggerSuffix}";
        
        var leftAnchorDuration = GetAnchorDuration(leftTrigger);
        var rightAnchorDuration = GetAnchorDuration(rightTrigger);

        return new CanonicalDelta
        {
            Id = DeltaIds.StructuralEmergence,
            Name = "Structure",
            Explanation = explanation,
            SummarySentence = $"{earlier} achieved structural dominance {stepDiff} steps before the other",
            Status = DeltaStatus.Present,
            LeftValue = leftDominanceTime,
            RightValue = rightDominanceTime,
            Delta = rightDominanceTime - leftDominanceTime,
            Magnitude = Math.Abs(rightDominanceTime - leftDominanceTime),
            VisualAnchorTime = Math.Min(leftDominanceTime, rightDominanceTime),
            DeltaType = DeltaType.Timing,
            Anchors = [new VisualAnchor
            {
                TargetView = "eigenvalues",
                RangeA = (leftDominanceStep, leftDominanceStep + leftAnchorDuration),
                RangeB = (rightDominanceStep, rightDominanceStep + rightAnchorDuration)
            }],
            TdA = leftDominanceStep,
            TdB = rightDominanceStep,
            DominanceRatioK = config.K
        };
    }

    /// <summary>
    /// Δ4. Evaluator Alignment Delta - "Did internal evaluators agree differently?"
    /// Phase 3.2: Persistence-weighted delta replaces mean-based delta.
    /// Rewards duration + consistency, not just amplitude.
    /// </summary>
    private static CanonicalDelta DetectEvaluatorAlignment(
        GeometryRun leftRun,
        GeometryRun rightRun,
        AlignmentDetectionConfig config,
        double currentTime,
        AlignmentMap alignment)
    {
        // Phase 3.2: Compute persistence-weighted alignment delta on the alignment map.
        var (persistenceScore, sustainedSegment, leftMean, rightMean, rangeA, rangeB, paired) = 
            ComputePersistenceWeightedAlignmentDelta(leftRun, rightRun, currentTime, config, alignment);
        if (paired == 0)
        {
            return new CanonicalDelta
            {
                Id = DeltaIds.EvaluatorAlignment,
                Name = "Agreement",
                Explanation = "No aligned samples",
                Status = DeltaStatus.Suppressed,
                Notes = ["Agreement was not compared where either side of a compare index is null"]
            };
        }

        var delta = rightMean - leftMean;
        var magnitude = persistenceScore; // Use persistence score as magnitude

        // Suppression: persistence-weighted difference is negligible
        // Phase 3.2: DeltaFloor lowered from 0.08 to 0.05, but now applies to persistence score
        if (magnitude < config.DeltaFloor)
        {
            return new CanonicalDelta
            {
                Id = DeltaIds.EvaluatorAlignment,
                Name = "Agreement",
                Explanation = "Similar evaluator alignment",
                Status = DeltaStatus.Suppressed,
                LeftValue = leftMean,
                RightValue = rightMean,
                Delta = delta,
                Magnitude = magnitude,
                Notes = ["Persistence-weighted alignment difference below threshold"],
                MeanAlignA = leftMean,
                MeanAlignB = rightMean
            };
        }

        // Suppression: sustained segment too short
        // Phase 3.2: Requires MinPersistenceSteps (default 4) sustained difference
        if (sustainedSegment.Duration < config.MinPersistenceSteps)
        {
            return new CanonicalDelta
            {
                Id = DeltaIds.EvaluatorAlignment,
                Name = "Agreement",
                Explanation = "Brief alignment difference",
                Status = DeltaStatus.Suppressed,
                LeftValue = leftMean,
                RightValue = rightMean,
                Delta = delta,
                Magnitude = magnitude,
                Notes = [$"Sustained segment ({sustainedSegment.Duration} steps) below minimum ({config.MinPersistenceSteps})"],
                MeanAlignA = leftMean,
                MeanAlignB = rightMean
            };
        }

        var higher = delta > 0 ? "Path B" : "Path A";
        var explanation = $"{higher} maintained higher evaluator agreement over {sustainedSegment.Duration} steps";

        return new CanonicalDelta
        {
            Id = DeltaIds.EvaluatorAlignment,
            Name = "Agreement",
            Explanation = explanation,
            SummarySentence = $"{higher} showed stronger internal alignment (sustained {sustainedSegment.Duration} steps)",
            Status = DeltaStatus.Present,
            LeftValue = leftMean,
            RightValue = rightMean,
            Delta = delta,
            Magnitude = magnitude,
            VisualAnchorTime = sustainedSegment.Start / (double)Math.Max(1, alignment.IdxToStepA.Length),
            DeltaType = DeltaType.Structure,
            // Anchor time is a fraction of the aligned series, and each range is that run's steps.
            Anchors = [new VisualAnchor
            {
                TargetView = "eigenvalues",
                RangeA = rangeA,
                RangeB = rangeB
            }],
            MeanAlignA = leftMean,
            MeanAlignB = rightMean
        };
    }

    /// <summary>
    /// Δ5. Stability / Oscillation Delta - "Did one run wobble more?"
    /// Phase 3.2: Area-above-θ scoring with stabilized adaptive threshold.
    /// </summary>
    private static CanonicalDelta DetectStabilityOscillation(
        GeometryRun leftRun,
        GeometryRun rightRun,
        StabilityConfig config,
        double currentTime,
        AlignmentMap alignment)
    {
        var leftLength = leftRun.Trajectory?.Timesteps?.Count ?? 0;
        var rightLength = rightRun.Trajectory?.Timesteps?.Count ?? 0;
        var pairedA = PairedSteps(alignment.IdxToStepA, alignment.IdxToStepB, leftLength, currentTime);
        var pairedB = PairedSteps(alignment.IdxToStepB, alignment.IdxToStepA, rightLength, currentTime);
        if (pairedA.Count == 0 || pairedB.Count == 0)
        {
            return new CanonicalDelta
            {
                Id = DeltaIds.StabilityOscillation,
                Name = "Stability",
                Explanation = "No aligned samples",
                Status = DeltaStatus.Suppressed,
                Notes = ["Oscillation was not compared where either side of a compare index is null"]
            };
        }

        var leftSlice = SliceToSteps(leftRun, pairedA);
        var rightSlice = SliceToSteps(rightRun, pairedB);
        var (leftScore, leftEpisodes, leftTheta, leftPeakEpisode) = ComputeOscillationScoreWithArea(leftSlice, 1, config);
        var (rightScore, rightEpisodes, rightTheta, rightPeakEpisode) = ComputeOscillationScoreWithArea(rightSlice, 1, config);

        var delta = rightScore - leftScore;
        var magnitude = Math.Abs(delta);

        // Suppression: both scores below NoiseFloor
        if (leftScore < config.NoiseFloor && rightScore < config.NoiseFloor)
        {
            return new CanonicalDelta
            {
                Id = DeltaIds.StabilityOscillation,
                Name = "Stability",
                Explanation = "Both runs stable",
                Status = DeltaStatus.Suppressed,
                LeftValue = leftScore,
                RightValue = rightScore,
                Delta = delta,
                Magnitude = magnitude,
                Notes = ["Both runs maintained stable trajectories (scores below noise floor)"],
                ScoreA = leftScore,
                ScoreB = rightScore,
                ThresholdUsed = leftTheta,
                MinDurationUsed = config.MinDuration
            };
        }

        // Suppression: |ΔO| below DeltaFloor
        if (magnitude < config.DeltaFloor)
        {
            return new CanonicalDelta
            {
                Id = DeltaIds.StabilityOscillation,
                Name = "Stability",
                Explanation = "Similar oscillation levels",
                Status = DeltaStatus.Suppressed,
                LeftValue = leftScore,
                RightValue = rightScore,
                Delta = delta,
                Magnitude = magnitude,
                Notes = [$"Oscillation difference below floor (|Δ|={magnitude:F3} < {config.DeltaFloor})"],
                ScoreA = leftScore,
                ScoreB = rightScore,
                ThresholdUsed = leftTheta,
                MinDurationUsed = config.MinDuration
            };
        }

        // Identify which path showed more oscillation
        string explanation;
        var peakEpisode = leftScore > rightScore ? leftPeakEpisode : rightPeakEpisode;
        var higherScore = Math.Max(leftScore, rightScore);
        
        if (leftScore > rightScore)
        {
            explanation = "Path A showed sustained instability during training";
        }
        else
        {
            explanation = "Path B showed sustained instability during training";
        }

        // Add duration info if we have a peak episode
        if (peakEpisode.Duration > 0)
        {
            explanation += $" ({peakEpisode.Duration} steps)";
        }

        var ownerIsLeft = leftScore >= rightScore;
        var ownerPaired = ownerIsLeft ? pairedA : pairedB;
        var ownerLength = Math.Max(1, ownerIsLeft ? leftLength : rightLength);
        var ownerEpisode = ownerIsLeft ? leftPeakEpisode : rightPeakEpisode;
        var ownerStep = ownerEpisode.Start >= 0 && ownerEpisode.Start < ownerPaired.Count
            ? ownerPaired[ownerEpisode.Start]
            : -1;
        var visualTime = ownerStep >= 0 ? ownerStep / (double)ownerLength : 0.5;
        (int, int)? RangeFor(List<int> paired, (int Start, int Duration, double Score) episode)
        {
            if (episode.Start < 0 || episode.Start >= paired.Count || episode.Duration <= 0)
                return null;
            var end = Math.Min(paired.Count - 1, episode.Start + episode.Duration - 1);
            return (paired[episode.Start], paired[end]);
        }

        return new CanonicalDelta
        {
            Id = DeltaIds.StabilityOscillation,
            Name = "Stability",
            Explanation = explanation,
            SummarySentence = $"{(leftScore > rightScore ? "Path A" : "Path B")} showed {magnitude:F2} more oscillation",
            Status = DeltaStatus.Present,
            LeftValue = leftScore,
            RightValue = rightScore,
            Delta = delta,
            Magnitude = magnitude,
            VisualAnchorTime = visualTime,
            DeltaType = DeltaType.Behavior,
            Anchors = [new VisualAnchor
            {
                TargetView = "curvature",
                RangeA = leftScore > rightScore ? RangeFor(pairedA, leftPeakEpisode) : null,
                RangeB = rightScore > leftScore ? RangeFor(pairedB, rightPeakEpisode) : null
            }],
            ScoreA = leftScore,
            ScoreB = rightScore,
            ThresholdUsed = leftTheta,
            MinDurationUsed = config.MinDuration
        };
    }

    // ========================================================================
    // HELPER METHODS
    // ========================================================================

    /// <summary>
    /// Check for persistent failure conditions (not single spikes).
    /// </summary>
    private static bool HasPersistentFailure(GeometryRun run, out double failureTime, out string failureType)
    {
        failureTime = 0;
        failureType = "instability";

        // Annotated events use the same persistence bar as inferred spikes:
        // one event is not enough. Three recorded failures count as persistent.
        if (run.Failures is { Count: >= 3 })
        {
            var ordered = run.Failures.OrderBy(f => f.T).ToList();
            failureTime = ordered[2].T;
            failureType = string.IsNullOrEmpty(ordered[0].Category) ? "instability" : ordered[0].Category;
            return true;
        }

        // Check for divergence (trajectory norm explosion)
        var steps = run.Trajectory?.Timesteps;
        if (steps != null && steps.Count > 10)
        {
            int divergenceCount = 0;
            for (int i = 1; i < steps.Count; i++)
            {
                var currentVel = steps[i].VelocityMagnitude;
                var priorVel = steps[Math.Max(0, i - 5)].VelocityMagnitude;
                if (currentVel > 10 * priorVel && currentVel > 1.0)
                {
                    divergenceCount++;
                    if (divergenceCount >= 3)
                    {
                        failureTime = (i - 2) / (double)(steps.Count - 1);
                        failureType = "divergence";
                        return true;
                    }
                }
                else
                {
                    divergenceCount = 0;
                }
            }
        }

        // Check for eigenvalue collapse
        var eigenvalues = run.Geometry?.Eigenvalues;
        if (eigenvalues != null && eigenvalues.Count > 10)
        {
            int collapseCount = 0;
            for (int i = 0; i < eigenvalues.Count; i++)
            {
                var sum = eigenvalues[i].Values?.Sum() ?? 0;
                if (sum < 0.001)
                {
                    collapseCount++;
                    if (collapseCount >= 3)
                    {
                        failureTime = (i - 2) / (double)(eigenvalues.Count - 1);
                        failureType = "geometry collapse";
                        return true;
                    }
                }
                else
                {
                    collapseCount = 0;
                }
            }
        }

        return false;
    }

    /// <summary>
    /// Estimate when the trajectory stabilizes.
    /// Returns (step, normalizedTime, epsilonUsed), or (-1, -1, 0) if never converges.
    /// </summary>
    private static (int Step, double Time, double Epsilon) EstimateConvergenceTime(GeometryRun run, ConvergenceConfig config)
    {
        var (step, time, epsilon, _, _) = EstimateConvergenceTimeWithConfidence(run, config);
        return (step, time, epsilon);
    }

    /// <summary>
    /// Phase 3.2: Estimate convergence with confidence heuristics.
    /// Returns (step, normalizedTime, epsilonUsed, tailLength, violationCount).
    /// Convergence = signal stays within ε band for Window steps (Option A: signal level).
    /// </summary>
    private static (int Step, double Time, double Epsilon, int TailLength, int Violations) 
        EstimateConvergenceTimeWithConfidence(GeometryRun run, ConvergenceConfig config)
    {
        var steps = run.Trajectory?.Timesteps;
        if (steps == null || steps.Count < config.Window * 2)
            return (-1, -1, 0, 0, 0);

        // Compute adaptive epsilon: max(base, sigma * multiplier) - Phase 3.2: using max() not sum()
        var velocities = steps.Select(s => s.VelocityMagnitude).ToList();
        var robustSigma = AlignmentMapper.RobustSigma(velocities);
        var epsilon = Math.Max(config.Epsilon, robustSigma * config.EpsilonSigmaMultiplier);

        int convergenceStep = -1;
        double convergenceTime = -1;

        for (int i = config.Window; i < steps.Count - config.Window; i++)
        {
            var baseValue = velocities[i];
            bool stable = true;

            for (int j = 1; j <= config.Window && i + j < steps.Count; j++)
            {
                if (Math.Abs(velocities[i + j] - baseValue) >= epsilon)
                {
                    stable = false;
                    break;
                }
            }

            if (stable)
            {
                convergenceStep = i;
                convergenceTime = i / (double)(steps.Count - 1);
                break;
            }
        }

        if (convergenceStep < 0)
            return (-1, -1, epsilon, 0, 0);

        // Compute confidence metrics
        // Tail length: how many steps remain stable after convergence
        int tailLength = 0;
        var baseValueAtConvergence = velocities[convergenceStep];
        for (int i = convergenceStep + config.Window; i < steps.Count; i++)
        {
            if (Math.Abs(velocities[i] - baseValueAtConvergence) < epsilon)
                tailLength++;
            else
                break;
        }

        // Violation count: how many times signal exits ε band after initial convergence
        int violations = 0;
        bool inBand = true;
        for (int i = convergenceStep + 1; i < steps.Count; i++)
        {
            bool nowInBand = Math.Abs(velocities[i] - baseValueAtConvergence) < epsilon;
            if (inBand && !nowInBand)
                violations++;
            inBand = nowInBand;
        }

        return (convergenceStep, convergenceTime, epsilon, tailLength, violations);
    }

    /// <summary>
    /// Compute integral-based oscillation score.
    /// Returns (score, peakStep, thresholdUsed).
    /// </summary>
    private static (double Score, int PeakStep, double Threshold) ComputeOscillationScore(GeometryRun run, double upToTime, StabilityConfig config)
    {
        var (score, _, theta, peakEpisode) = ComputeOscillationScoreWithArea(run, upToTime, config);
        return (score, peakEpisode.Start, theta);
    }

    /// <summary>
    /// Phase 3.2: Compute oscillation score using area-above-θ method.
    /// θ_eff = max(median×ThetaMultiplier, sigma×ThetaSigmaMultiplier)
    /// Score = sum of episode scores (area above threshold)
    /// </summary>
    private static (double Score, List<(int Start, int Duration, double EpisodeScore)> Episodes, double ThetaEff, (int Start, int Duration, double Score) PeakEpisode)
        ComputeOscillationScoreWithArea(GeometryRun run, double upToTime, StabilityConfig config)
    {
        var steps = run.Trajectory?.Timesteps;
        if (steps == null || steps.Count < config.MinDuration + 2)
            return (0, [], 0, (-1, 0, 0));

        var maxIdx = (int)(upToTime * (steps.Count - 1));
        maxIdx = Math.Clamp(maxIdx, config.MinDuration, steps.Count - 1);

        // Get absolute curvatures
        var curvatures = steps.Take(maxIdx + 1).Select(s => Math.Abs(s.Curvature)).ToList();
        
        // Compute adaptive θ: max(median×mult, sigma×mult)
        var sorted = curvatures.OrderBy(x => x).ToList();
        var medianAbs = sorted[sorted.Count / 2];
        var sigmaAbs = AlignmentMapper.RobustSigma(curvatures);
        
        // Use configured theta or compute adaptive
        double thetaEff;
        if (config.Theta > 0)
        {
            thetaEff = config.Theta;
        }
        else
        {
            var thetaFromMedian = medianAbs * config.ThetaMultiplier;
            var thetaFromSigma = sigmaAbs * config.ThetaSigmaMultiplier;
            thetaEff = Math.Max(thetaFromMedian, thetaFromSigma);
            // Ensure a minimum threshold to avoid noise triggering
            thetaEff = Math.Max(thetaEff, 0.01);
        }

        // Build episodes: contiguous segments where |C| > θ for ≥ MinDuration
        var episodes = new List<(int Start, int Duration, double EpisodeScore)>();
        int episodeStart = -1;
        double currentArea = 0;
        int currentDuration = 0;

        for (int i = 0; i <= maxIdx; i++)
        {
            double excess = Math.Max(0, curvatures[i] - thetaEff);
            
            if (curvatures[i] > thetaEff)
            {
                if (episodeStart < 0) episodeStart = i;
                currentDuration++;
                currentArea += excess;
            }
            else
            {
                // Episode ended - check if it qualifies
                if (currentDuration >= config.MinDuration && currentArea >= config.NoiseFloor)
                {
                    episodes.Add((episodeStart, currentDuration, currentArea));
                }
                episodeStart = -1;
                currentDuration = 0;
                currentArea = 0;
            }
        }

        // Don't forget trailing episode
        if (currentDuration >= config.MinDuration && currentArea >= config.NoiseFloor)
        {
            episodes.Add((episodeStart, currentDuration, currentArea));
        }

        // Total score = sum of episode scores
        var totalScore = episodes.Sum(e => e.EpisodeScore);

        // Find peak episode (highest score)
        var peakEpisode = episodes.Count > 0 
            ? episodes.OrderByDescending(e => e.EpisodeScore).First()
            : (-1, 0, 0.0);

        return (totalScore, episodes, thetaEff, peakEpisode);
    }

    /// <summary>
    /// Compute mean evaluator alignment over time window.
    /// </summary>
    private static (double Mean, double Variance) ComputeMeanEvaluatorAlignment(GeometryRun run, double currentTime, AlignmentDetectionConfig config)
    {
        var eigenvalues = run.Geometry?.Eigenvalues;
        if (eigenvalues == null || eigenvalues.Count == 0)
            return (0, 0);

        var maxIdx = (int)(currentTime * (eigenvalues.Count - 1));
        maxIdx = Math.Clamp(maxIdx, 0, eigenvalues.Count - 1);

        var alignments = new List<double>();
        
        for (int i = 0; i <= maxIdx; i++)
        {
            var values = eigenvalues[i].Values;
            if (values == null || values.Count < config.MinEvaluators)
                continue;

            var sum = values.Sum();
            if (sum > 0.001)
            {
                alignments.Add(values[0] / sum);
            }
        }

        if (alignments.Count == 0)
            return (0, 0);

        var mean = alignments.Average();
        var variance = alignments.Select(a => (a - mean) * (a - mean)).Average();

        return (mean, variance);
    }

    /// <summary>
    /// Phase 3.2: Persistence-weighted alignment delta.
    /// Instead of mean(B) - mean(A), computes ∫|A_B(t) - A_A(t)| dt over longest sustained segment.
    /// Rewards duration + consistency, not just amplitude.
    /// </summary>
    private static (double Score, (int Start, int Duration) Segment, double LeftMean, double RightMean, (int, int)? RangeA, (int, int)? RangeB, int Paired) 
        ComputePersistenceWeightedAlignmentDelta(
            GeometryRun leftRun, 
            GeometryRun rightRun, 
            double currentTime, 
            AlignmentDetectionConfig config,
            AlignmentMap alignment)
    {
        var leftEigen = leftRun.Geometry?.Eigenvalues;
        var rightEigen = rightRun.Geometry?.Eigenvalues;
        
        if (leftEigen == null || rightEigen == null || leftEigen.Count == 0 || rightEigen.Count == 0)
            return (0, (0, 0), 0, 0, null, null, 0);

        var compareCount = Math.Min(alignment.IdxToStepA.Length, alignment.IdxToStepB.Length);
        var limit = compareCount <= 1 ? 0 : (int)(currentTime * (compareCount - 1));
        limit = Math.Clamp(limit, 0, Math.Max(0, compareCount - 1));

        // D(t) only where both sides of the compare index exist.
        var leftAlignments = new List<double>();
        var rightAlignments = new List<double>();
        var differences = new List<double>();
        var pairedCompare = new List<int>();

        for (int i = 0; i <= limit && i < compareCount; i++)
        {
            var stepA = alignment.IdxToStepA[i];
            var stepB = alignment.IdxToStepB[i];
            if (stepA is null || stepB is null)
                continue;
            if (stepA.Value < 0 || stepA.Value >= leftEigen.Count || stepB.Value < 0 || stepB.Value >= rightEigen.Count)
                continue;

            var leftValues = leftEigen[stepA.Value].Values;
            var rightValues = rightEigen[stepB.Value].Values;
            
            double leftAlignment = 0, rightAlignment = 0;
            
            if (leftValues != null && leftValues.Count >= config.MinEvaluators)
            {
                var leftSum = leftValues.Sum();
                if (leftSum > 0.001)
                    leftAlignment = leftValues[0] / leftSum;
            }
            
            if (rightValues != null && rightValues.Count >= config.MinEvaluators)
            {
                var rightSum = rightValues.Sum();
                if (rightSum > 0.001)
                    rightAlignment = rightValues[0] / rightSum;
            }
            
            leftAlignments.Add(leftAlignment);
            rightAlignments.Add(rightAlignment);
            differences.Add(Math.Abs(rightAlignment - leftAlignment));
            pairedCompare.Add(i);
        }

        if (differences.Count == 0)
            return (0, (0, 0), 0, 0, null, null, 0);

        // Find longest sustained segment where D(t) > SegmentEpsilon
        int longestStart = 0;
        int longestDuration = 0;
        double longestAreaUnderCurve = 0;
        
        int currentStart = -1;
        int currentDuration = 0;
        double currentArea = 0;

        for (int i = 0; i < differences.Count; i++)
        {
            if (differences[i] > config.SegmentEpsilon)
            {
                if (currentStart < 0) currentStart = i;
                currentDuration++;
                currentArea += differences[i];
                
                if (currentDuration > longestDuration)
                {
                    longestStart = currentStart;
                    longestDuration = currentDuration;
                    longestAreaUnderCurve = currentArea;
                }
            }
            else
            {
                currentStart = -1;
                currentDuration = 0;
                currentArea = 0;
            }
        }

        // Persistence score = area under curve of longest sustained segment, normalized
        var persistenceScore = longestDuration > 0 
            ? longestAreaUnderCurve / longestDuration  // Average difference over sustained segment
            : 0;

        var leftMean = leftAlignments.Count > 0 ? leftAlignments.Average() : 0;
        var rightMean = rightAlignments.Count > 0 ? rightAlignments.Average() : 0;

        var compareStart = pairedCompare[longestStart];
        var compareEnd = pairedCompare[Math.Min(pairedCompare.Count - 1, longestStart + Math.Max(longestDuration, 1) - 1)];
        (int, int)? rangeA = null;
        (int, int)? rangeB = null;
        if (longestDuration > 0)
        {
            var a0 = alignment.IdxToStepA[compareStart];
            var a1 = alignment.IdxToStepA[compareEnd];
            var b0 = alignment.IdxToStepB[compareStart];
            var b1 = alignment.IdxToStepB[compareEnd];
            if (a0 is int startA && a1 is int endA)
                rangeA = (startA, endA);
            if (b0 is int startB && b1 is int endB)
                rangeB = (startB, endB);
        }

        return (persistenceScore, (compareStart, longestDuration), leftMean, rightMean, rangeA, rangeB, pairedCompare.Count);
    }

    /// <summary>
    /// Convergence anchor used by ΔTc and by temporal alignment.
    /// Time is -1 when the run never settles.
    /// </summary>
    public static (int Step, double Time) DetectConvergenceAnchor(GeometryRun run)
    {
        var (step, time, _) = EstimateConvergenceTime(run, DefaultConfig.Convergence);
        return (step, time);
    }

    private static List<int> PairedSteps(int?[] mine, int?[] other, int ownerLength, double upToTime)
    {
        var paired = new List<int>();
        var count = Math.Min(mine.Length, other.Length);
        var maxStep = ownerLength <= 1 ? 0 : (int)(Math.Clamp(upToTime, 0, 1) * (ownerLength - 1));
        for (int i = 0; i < count; i++)
        {
            if (mine[i] is not int step || other[i] is null)
                continue;
            if (step < 0 || step > maxStep)
                continue;
            paired.Add(step);
        }
        return paired;
    }

    private static GeometryRun SliceToSteps(GeometryRun run, List<int> steps)
    {
        var source = run.Trajectory?.Timesteps;
        if (source == null)
            return run;
        var chosen = new List<TrajectoryTimestep>();
        foreach (var step in steps)
        {
            if (step >= 0 && step < source.Count)
                chosen.Add(source[step]);
        }
        return run with { Trajectory = run.Trajectory! with { Timesteps = chosen } };
    }

    /// <summary>
    /// Phase 3.2: Find when first eigenvalue dominates using recurrence rule.
    /// Accepts emergence if EITHER:
    /// - Condition A: Sustained dominance (≥Window consecutive steps)
    /// - Condition B: Recurrence (≥2 segments of ≥2 steps within rolling window)
    /// Returns (step, normalizedTime, trigger), or (-1, -1, null) if never dominates.
    /// </summary>
    private static (int Step, double Time, string? Trigger) FindDominanceOnsetTimeWithRecurrence(
        GeometryRun run, 
        EmergenceConfig config)
    {
        var eigenvalues = run.Geometry?.Eigenvalues;
        if (eigenvalues == null || eigenvalues.Count < config.MinSegmentLength)
            return (-1, -1, null);

        // Step 1: Identify all dominance segments (≥MinSegmentLength consecutive steps)
        var segments = new List<(int Start, int Length)>();
        int segmentStart = -1;
        int segmentLength = 0;

        for (int i = 0; i < eigenvalues.Count; i++)
        {
            var values = eigenvalues[i].Values;
            bool isDominant = false;

            if (values != null && values.Count >= 2)
            {
                var lambda1 = values[0];
                var lambda2 = values[1];
                isDominant = lambda2 > 0.001 && lambda1 > config.K * lambda2;
            }

            if (isDominant)
            {
                if (segmentStart < 0) segmentStart = i;
                segmentLength++;
            }
            else
            {
                if (segmentLength >= config.MinSegmentLength)
                {
                    segments.Add((segmentStart, segmentLength));
                }
                segmentStart = -1;
                segmentLength = 0;
            }
        }
        // Don't forget trailing segment
        if (segmentLength >= config.MinSegmentLength)
        {
            segments.Add((segmentStart, segmentLength));
        }

        if (segments.Count == 0)
            return (-1, -1, null);

        // Step 2: Check Condition A — Sustained Dominance
        foreach (var seg in segments)
        {
            if (seg.Length >= config.Window)
            {
                var time = seg.Start / (double)(eigenvalues.Count - 1);
                return (seg.Start, time, "sustained");
            }
        }

        // Step 3: Check Condition B — Recurrence Rule
        // Need ≥MinRecurrenceCount segments within rolling RecurrenceWindow
        for (int i = 0; i < segments.Count; i++)
        {
            var windowStart = segments[i].Start;
            var windowEnd = windowStart + config.RecurrenceWindow;
            
            var segmentsInWindow = segments
                .Where(s => s.Start >= windowStart && s.Start < windowEnd)
                .ToList();

            if (segmentsInWindow.Count >= config.MinRecurrenceCount)
            {
                // Recurrence detected! Return start of first contributing segment
                var time = segmentsInWindow[0].Start / (double)(eigenvalues.Count - 1);
                return (segmentsInWindow[0].Start, time, "recurrence");
            }
        }

        // No emergence detected
        return (-1, -1, null);
    }

    /// <summary>
    /// Find when first eigenvalue dominates (legacy wrapper).
    /// Returns (step, normalizedTime), or (-1, -1) if never dominates.
    /// </summary>
    private static (int Step, double Time) FindDominanceOnsetTime(GeometryRun run, EmergenceConfig config)
    {
        var (step, time, _) = FindDominanceOnsetTimeWithRecurrence(run, config);
        return (step, time);
    }

    /// <summary>
    /// Find the step with maximum alignment difference between runs.
    /// </summary>
    private static int FindMaxAlignmentDifferenceStep(GeometryRun leftRun, GeometryRun rightRun, double currentTime)
    {
        var leftEigen = leftRun.Geometry?.Eigenvalues;
        var rightEigen = rightRun.Geometry?.Eigenvalues;
        
        if (leftEigen == null || rightEigen == null || leftEigen.Count == 0 || rightEigen.Count == 0)
            return -1;

        var minCount = Math.Min(leftEigen.Count, rightEigen.Count);
        var maxIdx = (int)(currentTime * (minCount - 1));
        maxIdx = Math.Clamp(maxIdx, 0, minCount - 1);

        double maxDiff = 0;
        int maxDiffStep = -1;

        for (int i = 0; i <= maxIdx; i++)
        {
            var leftValues = leftEigen[i].Values;
            var rightValues = rightEigen[i].Values;
            
            if (leftValues == null || rightValues == null || leftValues.Count < 2 || rightValues.Count < 2)
                continue;

            var leftSum = leftValues.Sum();
            var rightSum = rightValues.Sum();
            
            if (leftSum > 0.001 && rightSum > 0.001)
            {
                var leftAlignment = leftValues[0] / leftSum;
                var rightAlignment = rightValues[0] / rightSum;
                var diff = Math.Abs(leftAlignment - rightAlignment);
                
                if (diff > maxDiff)
                {
                    maxDiff = diff;
                    maxDiffStep = i;
                }
            }
        }

        return maxDiffStep;
    }

    /// <summary>
    /// Generate auto-summary from deltas following composition rules.
    /// </summary>
    public static string GenerateAutoSummary(List<CanonicalDelta> deltas)
    {
        if (deltas.Count == 0)
            return "No meaningful divergence observed between paths.";

        var primary = deltas[0];
        
        if (deltas.Count == 1)
            return primary.SummarySentence ?? $"{primary.Explanation}.";

        // Add modifier from second delta
        var secondary = deltas[1];
        var modifier = DeltaIds.Canonical(secondary.Id) switch
        {
            DeltaIds.ConvergenceTiming => " with different convergence timing",
            DeltaIds.StabilityOscillation => " alongside stability differences",
            DeltaIds.StructuralEmergence => " and distinct structural emergence",
            DeltaIds.EvaluatorAlignment => " while evaluator alignment differed",
            DeltaIds.FailurePresence => " compounded by failure events",
            _ => ""
        };

        var primaryText = primary.SummarySentence ?? primary.Explanation;
        var summary = $"{primaryText}{modifier}.";
        
        // Ensure we don't exceed 25 words
        var words = summary.Split(' ', StringSplitOptions.RemoveEmptyEntries);
        if (words.Length > 25)
        {
            return primary.SummarySentence ?? $"{primary.Explanation}.";
        }

        return summary;
    }
}
