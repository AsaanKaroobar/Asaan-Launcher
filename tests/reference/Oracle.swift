// Test-only adapters around the pinned, unmodified fuzzy matcher and ranking bodies.
// They substitute domain values, English section strings and stable Algorithms helpers.
import Foundation
struct RootItem: Codable { let id: String; let title: String; let isApp: Bool; let keywords: [String] }
struct RootResult: Codable { let item: RootItem; let section: String? }
extension Collection {
    func min(count: Int, by order: (Element, Element) -> Bool) -> [Element] {
        enumerated().sorted { a, b in order(a.element,b.element) || (!order(b.element,a.element) && a.offset < b.offset) }.prefix(count).map(\.element)
    }
    func partitioned(by predicate: (Element) -> Bool) -> ([Element], [Element]) {
        (filter { !predicate($0) }, filter(predicate))
    }
}
struct Pair: Codable { let query: String; let candidate: String }
struct RankingInput: Codable { let name: String; let mode: String; let query: String; let items: [RootItem]; let searchOnly: [RootItem]; let favorites: [String]; let aliases: [String:String]; let usage: [String:Double]; let limit: Int }
struct Input: Codable { let matches: [Pair]; let rankings: [RankingInput] }
struct MatchOutput: Codable { let score: Int; let matched: [Int] }
struct PairOutput: Codable { let query: String; let candidate: String; let expected: MatchOutput? }
struct RankingOutput: Codable { let input: RankingInput; let expected: [RootResult] }
struct Output: Codable { let matches: [PairOutput]; let rankings: [RankingOutput] }
let input = try JSONDecoder().decode(Input.self, from: FileHandle.standardInput.readDataToEndOfFile())
let matches = input.matches.map { pair in
    PairOutput(query: pair.query, candidate: pair.candidate, expected: Fuzzy.match(pair.query,pair.candidate).map { MatchOutput(score:$0.score,matched:$0.matched) })
}
let rankings = input.rankings.map { value in
    let usage: (String) -> Double = { value.usage[$0] ?? 0 }
    let result = value.mode == "browse"
      ? Ranking.browse(value.items, searchOnly:value.searchOnly, favorites:value.favorites, frecency:usage)
      : Ranking.search(value.items, query:value.query, favorites:value.favorites, alias:{ value.aliases[$0.id] }, frecency:usage, limit:value.limit)
    return RankingOutput(input:value, expected:result)
}
let encoder=JSONEncoder(); encoder.outputFormatting=[.sortedKeys]
FileHandle.standardOutput.write(try encoder.encode(Output(matches:matches,rankings:rankings)))
