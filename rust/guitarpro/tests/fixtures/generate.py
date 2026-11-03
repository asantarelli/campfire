# This file is part of Campfire.
# Copyright (c) Pandula Péter 2017-2026.
# https://github.com/pandulapeter/campfire
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
# If a copy of the MPL was not distributed with this file, You can obtain one at
# https://mozilla.org/MPL/2.0/.

"""Writes the Guitar Pro files the reader is tested against, and the JSON of what each one holds.

The files are written by PyGuitarPro (`pip install pyguitarpro`), an implementation of the format that is not ours, so that what
the reader makes of them is compared with what somebody else made of the same bytes: `python generate.py` here rewrites
`song.gp3`, `song.gp4`, `song.gp5` (5.00) and `song51.gp5` (5.10) and their `.json`. The JSON holds, per track and measure, the
beats of the first voice as `[start, value, is_dotted, enters, times, status, [[string, fret, type]...]]`.
"""

import json
import os

import guitarpro as gp

HERE = os.path.dirname(os.path.abspath(__file__))


def beat(voice, value, notes=(), dotted=False, tuplet=(1, 1), rest=False, text=None, effects=None):
    b = gp.Beat(voice)
    b.status = gp.BeatStatus.rest if rest else gp.BeatStatus.normal
    b.duration = gp.Duration(value=value, isDotted=dotted, tuplet=gp.Tuplet(enters=tuplet[0], times=tuplet[1]))
    if text:
        b.text = text
    for string, fret, *extra in notes:
        note = gp.Note(b, value=fret, string=string, type=gp.NoteType.normal)
        if extra:
            extra[0](note)
        b.notes.append(note)
    voice.beats.append(b)
    return b


def hammer(note):
    note.effect.hammer = True


def slide(note):
    note.effect.slides = [gp.SlideType.shiftSlideTo]


def bend(note):
    note.effect.bend = gp.BendEffect(type=gp.BendType.bend, value=100, points=[gp.BendPoint(0, 0), gp.BendPoint(6, 4), gp.BendPoint(12, 4)])


def harmonic(note):
    note.effect.harmonic = gp.NaturalHarmonic()


def vibrato(note):
    note.effect.vibrato = True


def palm(note):
    note.effect.palmMute = True


def ghost(note):
    note.effect.ghostNote = True


def accent(note):
    note.effect.accentuatedNote = True


def dead(note):
    note.type = gp.NoteType.dead


def tie(note):
    note.type = gp.NoteType.tie


def grace(note):
    note.effect.grace = gp.GraceEffect(fret=2, duration=16, isOnBeat=False)


def build():
    song = gp.Song()
    song.title = 'Práctica 002'
    song.artist = 'Yo'
    song.album = 'Pruebas'
    song.words = 'Letra'
    song.music = 'Música'
    song.tempo = 100
    song.measureHeaders = []
    specs = [
        dict(num=4, den=4, marker='Intro'),
        dict(num=4, den=4, open=True),
        dict(num=4, den=4),
        dict(num=4, den=4, close=2),
        dict(num=3, den=4),
        dict(num=6, den=8),
        dict(num=4, den=4, double=True),
        dict(num=4, den=4),
    ]
    start = gp.Duration.quarterTime
    for number, spec in enumerate(specs, 1):
        header = gp.MeasureHeader(number=number, start=start)
        header.timeSignature = gp.TimeSignature(numerator=spec['num'], denominator=gp.Duration(value=spec['den']))
        header.isRepeatOpen = spec.get('open', False)
        header.repeatClose = spec.get('close', -1)
        header.hasDoubleBar = spec.get('double', False)
        if 'marker' in spec:
            header.marker = gp.Marker(title=spec['marker'])
        song.measureHeaders.append(header)
        start += header.length
    song.tracks = []
    for number, (name, tuning) in enumerate([('Guitarra', [64, 59, 55, 50, 45, 40]), ('Bajo', [43, 38, 33, 28])], 1):
        track = gp.Track(song, number=number, name=name)
        track.strings = [gp.GuitarString(i + 1, value) for i, value in enumerate(tuning)]
        track.offset = 2 if number == 1 else 0
        track.measures = []
        for header in song.measureHeaders:
            measure = gp.Measure(track, header)
            track.measures.append(measure)
        song.tracks.append(track)

    guitar, bass = song.tracks
    voice = lambda track, index: track.measures[index].voices[0]
    # 1: quarters with a text and a chord
    v = voice(guitar, 0)
    beat(v, 4, [(5, 0), (4, 2)], text='Verso')
    beat(v, 4, [(3, 2)])
    beat(v, 4, [(2, 1, hammer)])
    beat(v, 4, rest=True)
    # 2: eighths, a dotted note and a rest
    v = voice(guitar, 1)
    for fret in (0, 2, 3, 2):
        beat(v, 8, [(4, fret)])
    beat(v, 4, [(3, 0, slide), (2, 0)], dotted=False)
    beat(v, 8, [(3, 2)], dotted=True)
    beat(v, 16, [(3, 3)])
    # 3: triplets and a chord, a tie, a dead note
    v = voice(guitar, 2)
    for fret in (0, 2, 4):
        beat(v, 8, [(2, fret)], tuplet=(3, 2))
    beat(v, 4, [(6, 3), (5, 2), (4, 0), (3, 0), (2, 3), (1, 3)])
    beat(v, 4, [(2, 3, tie)])
    beat(v, 4, [(3, 0, dead)])
    # 4: effects
    v = voice(guitar, 3)
    beat(v, 4, [(2, 5, bend)])
    beat(v, 4, [(3, 7, harmonic)])
    beat(v, 4, [(4, 0, vibrato)])
    beat(v, 4, [(5, 3, palm), (6, 3, accent)])
    # 5: 3/4 with a ghost note and a grace note
    v = voice(guitar, 4)
    beat(v, 4, [(1, 0, ghost)])
    beat(v, 4, [(2, 1, grace)])
    beat(v, 2, [(3, 0)])
    # 6: 6/8
    v = voice(guitar, 5)
    for fret in (0, 3, 5, 0, 3, 5):
        beat(v, 8, [(3, fret)])
    # 7: a whole note
    v = voice(guitar, 6)
    beat(v, 1, [(6, 0), (3, 2)])
    # 8: a half note and two quarters
    v = voice(guitar, 7)
    beat(v, 2, [(2, 0)])
    beat(v, 4, [(2, 1)])
    beat(v, 4, [(2, 3)])
    # the bass
    for index, header in enumerate(song.measureHeaders):
        v = voice(bass, index)
        count = header.timeSignature.numerator * (4 // header.timeSignature.denominator.value) if header.timeSignature.denominator.value in (4,) else 6
        value = 4 if header.timeSignature.denominator.value == 4 else 8
        for _ in range(count):
            beat(v, value, [(3, 0)])
    return song


def dump(song):
    tracks = []
    for track in song.tracks:
        measures = []
        for measure in track.measures:
            beats = []
            for b in measure.voices[0].beats:
                beats.append([
                    b.start, b.duration.value, b.duration.isDotted, b.duration.tuplet.enters, b.duration.tuplet.times, b.status.value,
                    [[n.string, n.value, n.type.value] for n in b.notes],
                ])
            measures.append(beats)
        tracks.append({'name': track.name, 'strings': [s.value for s in track.strings], 'capo': track.offset, 'measures': measures})
    headers = [
        {'numerator': h.timeSignature.numerator, 'denominator': h.timeSignature.denominator.value, 'repeat_open': h.isRepeatOpen,
         'repeat_close': h.repeatClose, 'marker': h.marker.title if h.marker else None, 'double_bar': h.hasDoubleBar}
        for h in song.measureHeaders
    ]
    return {'title': song.title, 'artist': song.artist, 'album': song.album, 'tempo': song.tempo, 'headers': headers, 'tracks': tracks}


def main():
    song = build()
    for name, version in [('song.gp3', (3, 0, 0)), ('song.gp4', (4, 0, 6)), ('song.gp5', (5, 0, 0)), ('song51.gp5', (5, 1, 0))]:
        path = os.path.join(HERE, name)
        gp.write(song, path, version=version)
        parsed = gp.parse(path)
        with open(path + '.json', 'w', encoding='utf-8') as out:
            json.dump(dump(parsed), out, ensure_ascii=False, indent=None)
        print(name, os.path.getsize(path), 'bytes')


if __name__ == '__main__':
    main()
