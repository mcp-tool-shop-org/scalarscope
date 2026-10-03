using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;

namespace VortexKit.Core;

/// <summary>
/// Shared playback controller for synchronized time-based visualization.
/// Multiple views bind to the same controller instance.
/// </summary>
public partial class PlaybackController : ObservableObject, IDisposable
{
    private System.Timers.Timer? _playbackTimer;
    private int _tickQueued;
    private int _disposed;
    private bool _keepCustomSpeed;
    private bool _hasLastSignal;
    private DateTime _lastSignalTime;
    private const double DefaultTickInterval = 16.67; // ~60fps
    private const double MinSpeed = 0.1;
    private const double MaxSpeed = 10.0;

    private static readonly double[] SpeedPresets = [0.25, 0.5, 1.0, 2.0, 4.0];

    /// <summary>
    /// Current playback position (0.0 to 1.0).
    /// </summary>
    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(TimePercent))]
    [NotifyPropertyChangedFor(nameof(TimeDisplay))]
    private double _time;

    /// <summary>
    /// Playback speed multiplier. Assigned values are clamped to [0.1, 10].
    /// </summary>
    [ObservableProperty]
    private double _speed = 1.0;

    /// <summary>
    /// Index into speed presets (0=0.25x, 1=0.5x, 2=1x, 3=2x, 4=4x).
    /// </summary>
    [ObservableProperty]
    private int _speedIndex = 2;

    /// <summary>
    /// Whether playback is currently active.
    /// </summary>
    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(IsNotPlaying))]
    private bool _isPlaying;

    /// <summary>
    /// Duration in seconds for a full playthrough at 1x speed. Must be positive.
    /// </summary>
    [ObservableProperty]
    private double _duration = 10.0;

    /// <summary>
    /// Whether playback should loop.
    /// </summary>
    [ObservableProperty]
    private bool _loop;

    public double TimePercent => Time * 100;
    public string TimeDisplay => $"{Time:P0}";
    public bool IsNotPlaying => !IsPlaying;

    /// <summary>
    /// Fired whenever time changes, including playback, a direct assignment, and a restart from the end.
    /// </summary>
    public event Action? TimeChanged;

    /// <summary>
    /// Fired when playback reaches the end.
    /// </summary>
    public event Action? PlaybackEnded;

    public PlaybackController()
    {
        _playbackTimer = new System.Timers.Timer(DefaultTickInterval);
        _playbackTimer.Elapsed += OnPlaybackTick;
    }

    partial void OnTimeChanged(double value)
    {
        if (!double.IsFinite(value) || value < 0.0 || value > 1.0)
        {
            Time = Math.Clamp(double.IsFinite(value) ? value : 0.0, 0.0, 1.0);
            return;
        }

        TimeChanged?.Invoke();
    }

    partial void OnSpeedChanged(double value)
    {
        if (double.IsFinite(value) && value >= MinSpeed && value <= MaxSpeed)
            return;

        Speed = ClampSpeed(value);
    }

    partial void OnDurationChanging(double value)
    {
        if (!(value > 0.0) || !double.IsFinite(value))
            throw new ArgumentOutOfRangeException(nameof(Duration), "Duration must be positive.");
    }

    partial void OnSpeedIndexChanged(int value)
    {
        // SetSpeed updates the nearest-preset index without replacing a custom speed.
        if (_keepCustomSpeed)
            return;

        if ((uint)value < (uint)SpeedPresets.Length)
            Speed = SpeedPresets[value];
    }

    [RelayCommand]
    public void PlayPause()
    {
        IsPlaying = !IsPlaying;

        if (IsPlaying)
        {
            // Reset to start if at end. The Time setter raises TimeChanged.
            if (Time >= 1.0)
                Time = 0;

            _hasLastSignal = false;
            _playbackTimer?.Start();
        }
        else
        {
            _playbackTimer?.Stop();
            _hasLastSignal = false;
        }
    }

    [RelayCommand]
    public void Stop()
    {
        IsPlaying = false;
        _playbackTimer?.Stop();
        _hasLastSignal = false;
        Time = 0;
    }

    [RelayCommand]
    public void StepForward(double? stepSize = null)
    {
        var step = stepSize ?? 0.01;
        Time = Math.Min(1.0, Time + step);
    }

    [RelayCommand]
    public void StepBackward(double? stepSize = null)
    {
        var step = stepSize ?? 0.01;
        Time = Math.Max(0.0, Time - step);
    }

    [RelayCommand]
    public void JumpToTime(double t)
    {
        Time = Math.Clamp(double.IsFinite(t) ? t : 0.0, 0.0, 1.0);
    }

    [RelayCommand]
    public void JumpToStart() => JumpToTime(0.0);

    [RelayCommand]
    public void JumpToEnd() => JumpToTime(1.0);

    public void IncreaseSpeed()
    {
        if (SpeedIndex < SpeedPresets.Length - 1)
            SpeedIndex++;
    }

    public void DecreaseSpeed()
    {
        if (SpeedIndex > 0)
            SpeedIndex--;
    }

    /// <summary>
    /// Sets speed to the requested value clamped into [0.1, 10].
    /// The same input always ends at that speed. SpeedIndex moves to the nearest preset and does not replace it.
    /// </summary>
    public void SetSpeed(double speed)
    {
        var clamped = ClampSpeed(speed);
        Speed = clamped;

        var closestIdx = 0;
        var closestDist = double.MaxValue;
        for (var i = 0; i < SpeedPresets.Length; i++)
        {
            var dist = Math.Abs(SpeedPresets[i] - clamped);
            if (dist < closestDist)
            {
                closestDist = dist;
                closestIdx = i;
            }
        }

        if (SpeedIndex == closestIdx)
            return;

        _keepCustomSpeed = true;
        try
        {
            SpeedIndex = closestIdx;
        }
        finally
        {
            _keepCustomSpeed = false;
        }
    }

    private static double ClampSpeed(double speed)
    {
        if (!double.IsFinite(speed))
            return 1.0;
        return Math.Clamp(speed, MinSpeed, MaxSpeed);
    }

    private void OnPlaybackTick(object? sender, System.Timers.ElapsedEventArgs e)
    {
        // The timer thread must not write bound properties. Queue one UI tick and drop overlaps.
        if (Volatile.Read(ref _disposed) != 0)
            return;

        if (Interlocked.CompareExchange(ref _tickQueued, 1, 0) != 0)
            return;

        var signalTime = e.SignalTime;
        var queued = false;
        try
        {
            MainThread.BeginInvokeOnMainThread(() => AdvancePlaybackOnUi(signalTime));
            queued = true;
        }
        catch (InvalidOperationException)
        {
            // No UI thread in this process. Drop the tick.
        }
        finally
        {
            if (!queued)
                Interlocked.Exchange(ref _tickQueued, 0);
        }
    }

    private void AdvancePlaybackOnUi(DateTime signalTime)
    {
        try
        {
            if (Volatile.Read(ref _disposed) != 0 || !IsPlaying || _playbackTimer == null)
                return;

            double elapsedSeconds;
            if (!_hasLastSignal)
            {
                elapsedSeconds = DefaultTickInterval / 1000.0;
            }
            else
            {
                elapsedSeconds = (signalTime - _lastSignalTime).TotalSeconds;
                if (!double.IsFinite(elapsedSeconds) || elapsedSeconds < 0)
                    elapsedSeconds = 0;
            }

            _lastSignalTime = signalTime;
            _hasLastSignal = true;

            if (!(Duration > 0) || !(Speed > 0))
                return;

            var next = Time + elapsedSeconds * Speed / Duration;
            if (!double.IsFinite(next))
                return;

            if (next >= 1.0)
            {
                if (Loop)
                {
                    Time = 0.0;
                }
                else
                {
                    IsPlaying = false;
                    _hasLastSignal = false;
                    _playbackTimer?.Stop();
                    Time = 1.0;
                    PlaybackEnded?.Invoke();
                }
            }
            else if (next <= 0.0)
            {
                Time = 0.0;
            }
            else
            {
                Time = next;
            }
        }
        finally
        {
            Interlocked.Exchange(ref _tickQueued, 0);
        }
    }

    public void Dispose()
    {
        if (Interlocked.Exchange(ref _disposed, 1) != 0)
            return;

        var timer = _playbackTimer;
        _playbackTimer = null;
        _hasLastSignal = false;
        if (timer != null)
        {
            timer.Elapsed -= OnPlaybackTick;
            timer.Stop();
            timer.Dispose();
        }

        GC.SuppressFinalize(this);
    }
}
