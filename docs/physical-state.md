# Physical board and search projections

Frostetra follows the
[SBP/1 physical-state contract](https://github.com/ruelalarcon/stacker_bot_protocol/blob/master/spec/sbp-1.md#95-advance).
The client owns physical board truth. `start` supplies the initial board;
`advance` applies the accepted piece and line clears; `board` replaces physical
cells when the client observes garbage or another board correction.

`incoming_garbage` describes pending chunks. It does not identify arrival times
or holes and does not authorize adding rows to the current board. A client need
not resend an unchanged board after every accepted placement.

Search implementations may project garbage arrivals when evaluating future
moves. Such projections must remain in hypothetical descendants. Both the bot's
current state and the root from which it generates its next suggestion must
use the physical board after accepted placements. Promoting a speculative child
to the root must not carry an unconfirmed garbage rise with it.

For example, if five rows remain pending after a non-clearing lock, the next
suggestion must fit the unraised board. Only a physical board update confirms
the actual rise and hole positions. Receiving the same pending queue again does
not confirm that the previous prediction happened.

`tests/physical_board.rs` exercises this through SBP messages. It checks both
advancing without a rise and accepting a later authoritative board with a rise.
