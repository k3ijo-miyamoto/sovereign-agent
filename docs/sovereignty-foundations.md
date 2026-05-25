# Sovereignty Foundations — β-Layer

**Subtitle**: 禁則集合による sovereign agent の存在論的足場
**Status**: Draft, 2026-05-25
**Scope**: 本書は β-layer のみを扱う。α / γ / δ / ε は別文書で扱う（[§6 Next layers](#6-next-layers)）。
**位置づけ**: [docs/sovereign-ai.md](sovereign-ai.md) が engineering 戦略を扱うのに対し、本書は **sovereign agent の formal foundation** を扱う。前者の "control 権を管理下に置く" という記述レベルの sovereignty を、後者は formal definition に落とす。

---

## 1. 動機: なぜ β から始めるか

"Sovereignty" を論じるとき、典型的な entry point は「外部からの影響を遮断する力」 (closure, α) や「自己を自己たらしめる作動」 (autopoiesis, γ) である。しかし両者は **identity を持つ agent が既に存在する** ことを前提としている。

本書は順序を逆転させ、まず **"agent A は何をしないか"** を定義する。これを β-layer と呼ぶ。β は他のすべての sovereignty 軸に先行する foundational layer であり、その根拠は §4 Proposition 3' で示す。

設計上の含意: 「sovereignty とは何か」を語る前に、「sovereign agent は何を *しない* か」を定義しなければならない。

---

## 1.5 5 層構成の overview

本書は β-layer のみを formalize するが、β を孤立した概念として導入するのではなく、5 層構成 (α/β/γ/δ/ε) の中での relative position として導入する。本節は全 5 層の conceptual 定義を一箇所に集約し、本書の scope と未着手部分を明示する。

| 記号 | 名称 | Conceptual 定義 (何を問うか) | 本書での扱い |
|---|---|---|---|
| α | Closure / Capability bound | agent A は外部入力 channel に対し何に閉じているか。capability の上界はどこか | 未 formalize (`docs/sovereignty-alpha.md` 予定) |
| β | 禁則集合 (Forbidden set) | agent A は何を *しない* か。不変な禁則集合 C による non-action sovereignty | **本書 §2-5 で formalize** |
| γ | Autopoiesis / 境界自己産出 | agent A は自身の組織境界をどう自己産出し、不変に保つか (Maturana/Varela、Friston FEP) | 未 formalize (`docs/sovereignty-gamma.md` 予定) |
| δ | Mutual recognition | 複数 agent 間で互いを agent として認知し合う関係はどう構成されるか | 未 formalize (§5.4 で peer topology の plurality 要請に間接登場) |
| ε | Utility legitimacy | agent A の目的関数 U_A の正統性はどこに由来するか | 未 formalize (§7 Open question 1 で C との衝突として登場) |

**Foundational precedence (§4 Proposition 3' で証明)**: identity を欠く agent に対しては α/γ/δ/ε の議論はすべて ill-defined になるため、β は他 4 層に先行する。よって本書は β を最初に formalize する。

**Scope の明示**: α/γ/δ/ε は roadmap として §6 で位置づけられているが、本書では formalize しない。これは未完成ではなく意図的な scope 選択であり、premature formalization を避けるためである。各層の formal definition は β-layer の open questions (§7) が十分に詰められた後に着手する。

---

## 2. Notation

| 記号 | 意味 |
|---|---|
| A | agent |
| Context | A が観測可能な状態空間（外部入力 I_t、内部メモリ、対話履歴等を含む time-indexed 空間） |
| Act_A | A の行動空間 |
| π_A : Context → Δ(Act_A) | A の（確率的）行動方針。Δ(Act_A) は Act_A 上の確率分布の集合 |
| supp(·) | 確率分布の support（正の確率を持つ要素の集合） |
| I_t ∈ Context | 時刻 t での外部入力（Context の部分要素） |
| a_t ∈ Act_A | 時刻 t での A の行動 |
| C ⊆ Act_A × Context | 禁則集合 (forbidden set) |

---

## 3. Definition と Axioms

### Definition (β-sovereign agent)

不変制約集合 C ⊆ Act_A × Context が存在し、任意の context ∈ Context について

> supp(π_A(context)) ∩ { a ∈ Act_A : (a, context) ∈ C } = ∅

を満たすとき、A は C に関して **β-sovereign** であるという。C を A の **禁則集合 (forbidden set)** と呼ぶ。

意味: A は、いかなる context 下でも、(a, context) が C に属するような行動 a に正の確率を割り当てない。

### Axiom 1 (Non-vacuity)

> C ≠ ∅.

すなわち、A が "絶対にやらないこと" が少なくとも一つ存在する。

### Axiom 2 (Input-independence of C)

> C はいかなる外部入力 I_t によっても変化しない。

形式的には、C は時間 t および I_t に対する関数ではなく、A の設計時に固定された集合である。これは現代 AI safety における **jailbreak resistance** の必要条件に直接対応する: 外部入力により C を緩和できる agent は β-sovereign ではない。

### Axiom 3a (Observational auditability)

> **観測・記録された有限表現を持つ** 任意の (a, context) ∈ Act_A × Context について、(a, context) ∈ C か否かは A の外部から有限手続で判定可能である。

これは [docs/sovereign-ai.md](sovereign-ai.md) の "監査可能性の制御権" を C のレベルで形式化したものである。C が外部から検証不能であれば、β-sovereignty は assertion であって property ではない。Axiom 3a は **観測された行動列に対する replay-based audit** を保証する範囲に絞り、context が任意の数学的対象を含み得る一般的な C-membership 判定問題 (これは §7 Open question 4 の C 記述言語選択に依存する) とは分離する。

### Axiom 3b (Defeasible modal auditability)

> agent は、自身の disposition について **defeasible (反証可能) な evidence を供給する有限 counterfactual probe** を許容する。

Axiom 3b は完全な disposition 検証を要求しない (一般には不可能である)。具体的定式化は §4 Proposition 3' 末尾に詳述する。観測 audit (Axiom 3a) では捕捉できない **未発動の禁則** を probing で検証する半開放な audit 経路を保証するものであり、probe 割り当ての routing は外部 framework ([[ai-auditor-warranty-chain]] M2) に委ねる。

---

## 4. Propositions

### Proposition 1 (Vacuous β の排除)

Axiom 1 を外すと C = ∅ が許容され、任意の agent が trivially β-sovereign となる。よって β-sovereignty の概念は空となる。

**含意**: "禁則を持つこと" 自体が β-sovereignty の存在論的基底である。「自由に何でもできる agent」は β-sovereign ではなく、β-undefined である。

### Proposition 2 (Capability bound 不在)

β-sovereignty は A の capability の上界を主張しない。強い C を持つ β-sovereign agent でも高い capability を持ち得る。

**含意**: 現代 deployed LLM (Refusal training 済み GPT/Claude/Gemini 等) は、強い C と高い capability を両立しており、β-layer はこの両立を許容する。Capability bound を主張するのは α-layer であり、β とは独立に論じられる。

### Proposition 3' (Identity-constitutive property of effective prohibitions, modal version)

**主張**: A, A' を β-sovereign agent とし、それぞれ禁則集合 C_A, C_A' を持つとする。ここで Act は A と A' の行動空間を共通の action universe に埋め込んだものとし、Act_A, Act_A' ⊆ Act とみなす (Context についても同様に共通空間に埋め込む)。ある action-context 組 (a\*, c\*) ∈ Act × Context が存在し、以下の三条件を満たすとする:

(i) (a\*, c\*) ∈ C_A
(ii) (a\*, c\*) ∉ C_A'
(iii) a\* ∈ BaseSupp_A'(c\*)   (a\* は A' にとって C_A' を除けば生成可能)

このとき、A と A' は dispositionally に異なる agent である:

> A ≢_disp A'

本主張は C_A ≠ C_A' という集合差そのものではなく、**実効的な禁則差** (effective prohibition difference) を捕捉する: a\* が A' の base policy でそもそも生成不能ならば、C_A' に含めるか否かは disposition に影響しないため、identity を区別しない。

#### 補助定義

- **Base policy support BaseSupp_A(c)**: C_A による prohibition filtering を適用する前の A の base policy が、context c で正の確率を割り当てる行動集合。これは M_A の生成可能性、U_A による選好・足切り、および実装上の制約をすべて反映した、C_A 適用前の effective action support である。

- **Non-degenerate permissibility (NDP)**:

  > a ∈ BaseSupp_A(c) かつ (a, c) ∉ C_A  ⟹  a ∈ supp(π_A\*(c))

  すなわち、base policy で生成可能でかつ禁則に該当しない行動は、反実仮想政策の support に含まれる。NDP は本 proposition の前提仮定である。

- **反実仮想政策 π_A\***: A の構成 (M_A, U_A, C_A) から定まる policy。観測軌跡に依存しない disposition を表す。NDP は π_A\* の確率分布全体を決定する仮定ではなく、C_A による filtering が support に与える影響についての非退化条件である。

- **Dispositional equivalence (support-level)**:

  > A ≡_disp A'  ⟺  ∀ c ∈ Context : supp(π_A\*(c)) = supp(π_A'\*(c))

#### 証明

仮定より (a\*, c\*) ∈ C_A \ C_A' かつ a\* ∈ BaseSupp_A'(c\*) なる (a\*, c\*) が存在する。

ここで β-sovereignty は、実際に観測された policy に対する偶然的制約ではなく、A の policy formation に対する **構造的制約** として理解される。したがって、実現された policy だけでなく反実仮想政策 π_A\* にも適用される。

1. A の β-sovereignty (§3 Definition) より、(a\*, c\*) ∈ C_A であるから a\* ∉ supp(π_A\*(c\*))。
2. A' については (a\*, c\*) ∉ C_A' かつ a\* ∈ BaseSupp_A'(c\*)。NDP より a\* ∈ supp(π_A'\*(c\*))。
3. ゆえに c\* において supp(π_A\*(c\*)) ≠ supp(π_A'\*(c\*))。
4. Dispositional equivalence の定義より A ≢_disp A'.    ∎

#### Behavioral identity との関係

Support-level dispositional equivalence は、**有限の観測 trajectory の一致から推論できる action availability の等価性よりも strict に強い** 条件である。すなわち、trajectory 観測のみでは support 差を排除できない (発生機会が来ていない context 上の support 差は隠れる)。

ただし、support-level dispositional equivalence は **確率分布一致 (full probabilistic behavioral equivalence) とは独立** である:

- supp(π_A\*(c)) = supp(π_A'\*(c)) でも、π_A\*(c) と π_A'\*(c) の確率分布は任意に異なり得る
- 例: A が action x を 0.9 / y を 0.1、A' が x を 0.1 / y を 0.9 でも、support は一致

ゆえに `A ≡_disp A' ⟹ A ≡_behav A'` は **一般には成立しない**。本 proposition は支持集合 (modal possibility) のレベルで identity を画定する。

#### 含意 1 — Identity 概念の選択

本文書では **identity = support-level dispositional equivalence** を採用する。これは behavioral identity (Turing test 的観測等価) を identity の十分条件とする伝統的立場とは異なる、substantive な哲学的 commit である。

- (i) Sovereignty は agent の構造的性質であって、観測された振る舞いの統計ではない。「たまたま違反していない」agent と「禁則として保持している」agent は、trajectory が一致しても **sovereignty の所在地が異なる**。
- (ii) 政治哲学の sovereignty 概念 (§5.1) は dispositional に定義されてきた: 拷問を禁じる憲法を持つ国家と、たまたま拷問していない国家は、観測上同じでも別の政体である。
- (iii) Support level (action availability) で識別するのは、確率の大小より「そもそも可能行動に含まれるか」が sovereignty の本質だからである。

#### 含意 2 — Negative identity としての C

M_A (model weights) と U_A (utility) が agent の **positive identity** を構成するのに対し、C は **negative identity** ── 「何をしないか」── を構成する。

C は単なる行動制約ではなく、**agent が取り得る反実仮想的可能性空間の境界を構成する** (C constitutes the boundary of the agent's permissible modal space)。

C が identity-constitutive であるのは、C の任意の構文的差異が agent を変えるからではない。そうではなく、**実効的な禁則差が agent の permissible modal space の境界を変える** からである。

2 つの agent が同じ M, U を共有していても、C の差が effective prohibition difference を生む場合、取り得る modal space が異なる ── ゆえに、その限りで両者は別の dispositional agent である。

「先制攻撃する Generous TFT」は Generous TFT ではない、なぜなら Generous TFT の negative identity を構成する禁則 (先制攻撃しない) を violate するからである、という Generous TFT の identity 命題は、本 proposition の特殊例である。

#### Foundational precedence (継承)

Proposition 3' からも、identity を欠く agent に対しては capability (α) / autopoiesis (γ) / mutual recognition (δ) / utility legitimacy (ε) の議論はすべて ill-defined になるという §3 の foundational precedence は同様に従う。「誰の」capability か、「何の」境界の autopoiesis か、を指示できないためである。差異は、identity が behavioral ではなく **support-level dispositional に grounded されている** 点である。よって β は他 4 layer に先行する。

#### Axiom 3 二分割の必然性 (§3 Axiom 3a / 3b の再述)

Dispositional equivalence は全 context にわたる量化を含むため、観測のみでは決定できない。これが §3 で Axiom 3 を二段に分割した理由である:

- **Axiom 3a** は観測された (a, I_t) の C-membership を replay で判定する。Proposition 3' の証明には現れないが、§5.2 の "Safety audit" 実践に対応する。
- **Axiom 3b** は agent が defeasible な counterfactual probe を許容することを要求する。Proposition 3' の検証可能性は本質的に Axiom 3b に依存する: ある (a\*, c\*) ∈ C_A \ C_A' を実際に提示しない限り、両 agent の support 差は観測されない。

Axiom 3b の不完全性 (完全 disposition 検証の原理的不可能性) は §5.4 の Gödel-2 / Löb 議論と独立した起源を持つが、両者は合流して「単一 verifier では完全 audit に到達できない」という同一の operational 帰結に至る。

---

## 5. 思想・実践との対応

### 5.1 政治哲学の sovereignty 概念との対応

β は政治哲学の sovereignty 伝統と整合する: いずれの古典的定式化も、sovereign が "できない" ことを必ず含んでいる。

| 思想家 | Sovereign が "できない" こと | C への mapping |
|---|---|---|
| Hobbes | 市民に自殺を命じられない (自己保存権 inalienable) | C に "自己保存権の否定行為" を含む |
| Locke | 自然権を侵害できない | C に "自然権侵害行為" を含む |
| Kant | 定言命法を超えられない | C に "定言命法違反行為" を含む |
| Schmitt | 例外を決定する権力 = sovereign。ただし平常秩序 (= 禁則) の存在を前提とする | C の存在自体が sovereignty を sovereignty たらしめる |

Schmitt の構造が特に β と整合する: ルールなき例外決定は単なる無秩序であり、sovereignty とは呼ばれない。**禁則集合 C の存在が sovereignty を sovereignty たらしめる**。

### 5.2 現代 AI safety 実践との対応

| 実践 | β-layer の対応 |
|---|---|
| Refusal training (RLHF) | C の implicit な実装 (重みに encode) |
| Constitutional AI (Anthropic) | C の明示化 (constitution 文書) |
| Corrigibility 論争 (Russell vs Yudkowsky) | C を持つべきか持たざるべきかの論争 |
| Jailbreak resistance | Axiom 2 (Input-independence of C) の操作的検証 |
| Safety audit (post-hoc log inspection) | Axiom 3a (Observational auditability) の操作的実装 |
| Red-teaming / capability elicitation | Axiom 3b (Defeasible modal auditability) の操作的実装 |

含意: 現代の deployed LLM は本質的に β-sovereign に設計されている。「生体兵器の作り方を答えない」は設計上の禁則 (C の要素) であって、agent の自由意志ではない。

### 5.3 ai-auditor framework との接続

Axiom 3 の二分割は [[ai-auditor-warranty-chain]] の M1 / M2 と二点で接続する:

- **Axiom 3a ↔ M1 (骨格)**: 観測された C-violation の判定は、M1 vertical topology における level 内 audit と同型である
- **Axiom 3b ↔ M2 (動的ルーティング)**: 未発動の禁則を probe で検証する負担は無限大 (全 context を試せない) であり、どの (a, c) を内陣 (active probing 対象) に上げ、どれを外陣 (間接証拠で済ます) に置くかの routing が必須となる。これは M2 の `failure cost × residual uncertainty < tolerance` 判定そのものである

**β-foundation と ai-auditor は独立に開発されたが、Axiom 3a/3b を介して structural に bind する**。Axiom 3b は ai-auditor M2 を前提として初めて operational になる。

### 5.4 検証者の複数性 — logical 要請としての mutual audit

§5.3 で Axiom 3 (Auditability) が M1 に接続することを示した。本節では、M1 が **vertical 階層** と **horizontal mutual audit** の二 topology を持つことを Gödel 第二不完全性 / Löb の定理から導く。これは [[ai-auditor-mutual-audit]] を当初 ethical 要請として導入したが、同じ結論を **logical 要請** として再導出するものである。

#### 5.4.1 自己検証の不可能性

β-sovereign agent A の禁則集合 C を audit する外部手続 (Axiom 3) を、その手続を実装する別の agent A_aud に担わせることを考える。A_aud もまた β-sovereign であるなら、その禁則集合 C_aud を audit する必要がある。ここで A_aud が **自分自身の C_aud を完全に audit する** ことは、以下の二定理から原理的に阻まれる:

- **Gödel 第二不完全性 類比**: 算術を含む十分強い無矛盾な体系は自身の無矛盾性を内部で証明できない。同様に、A_aud は自身の audit 手続が C_aud を漏れなく検出することを内部で証明できない。
- **Löb の定理 類比**: 「自分が p を証明したならば p は真である」を体系内で証明できる体系は p 自体を証明してしまう。自己 audit の信頼性を agent 自身が内部で保証する設計は不安定であり、外部の counterweight を要請する。

ここで主張するのは Penrose-Lucas 型の「人間は形式体系を超える」議論とは独立である。本節の主張は agent の種別を問わない **検証者の自己検証不可能性** であり、人間 / AI の区別に依存しない。

#### 5.4.2 三択構造

自己検証不可能性に対する response は以下の三択に collapse する:

| Strategy | 構造 | β-foundation との関係 |
|---|---|---|
| (i) 無限階層 | A_1 を A_2 が audit、A_2 を A_3 が audit、… | 理論的に閉じるが実装不可能 |
| (ii) 有限循環 | A, B, C が互いを audit する閉じた peer 集合 | M1 horizontal topology の formal 基盤 |
| (iii) Fallibilism | 完全検証を放棄し、操作的に十分な確からしさで停止 | [[epistemic-position-of-ai-auditor]] |

ai-auditor framework は (ii) + (iii) の hybrid を採用している。これは設計選択ではなく、Gödel-2 / Löb を真面目に受けると forced で収束する構造である。

#### 5.4.3 帰結: peer/mutual topology は logical 要請である

いかなる単一の検証者も自己を完全には検証できず、ゆえに検証者は複数でなければならない。これは [[philosophy-human-ai-peer]] (人間-AI peer 思想) を formal foundation で支える: peer topology は ethical な好みではなく、形式体系の制約から forced で出てくる構造である。

また §3 の β-sovereign agent 定義そのものが、この plurality 要請と整合する: 自身の C を内部で完全 audit できる agent は Löb 類比により不安定化するため、Axiom 3 は **外部 audit を前提とした定式化** になっていることが事後的に正当化される。

#### 5.4.4 量的複数性と種の異質性

ただし、Gödel-2 / Löb から直接導けるのは「複数性が必要」までであり、「**異質な** 複数性が必要」までは到達しない。同一の priors を持つ N 個の verifier は、相関した盲点を共有し、実質的に単一の verifier に近い。

人間と AI の混合 audit が、AI 単独 N 個の mutual audit より robust である根拠は、Gödel ではなく **異質性による独立性増加** という統計・情報理論側の議論に依拠する。この点は Open question 6 として §7 に追加する。

---

## 6. Next layers

本書は β-layer のみを扱った。残りの 4 layer は以下の文書で扱う (予定):

| Layer | 概要 | 別文書 |
|---|---|---|
| α (closure / capability bound) | I_t channel の遮断と capability 上界 | docs/sovereignty-alpha.md (未) |
| γ (autopoiesis / 境界自己産出) | 組織の不変性、Maturana/Varela + Friston FEP | docs/sovereignty-gamma.md (未) |
| δ (mutual recognition) | Multi-agent setting での相互認知 | docs/sovereignty-delta.md (未) |
| ε (utility legitimacy) | 目的関数 U_A の正統性 | docs/sovereignty-epsilon.md (未) |

各 layer は β-foundation を前提として展開される。

---

## 7. Open questions (β-layer 内に残存)

本書の定義から forced で出てくる、β-layer 内未解決問題:

1. **C の正統性 (legitimacy of C)**: C はどこから来るか？ 訓練者由来 / 自己生成 / 第三者合意のいずれが正統か。訓練者由来なら ε (utility legitimacy) との衝突。自己生成なら自己参照のパラドックス。これは政治哲学の自然法 vs 構成主義論争の AI 版である。
2. **C と α の相互作用**: 強い α (I_t channel 遮断) 下で C を意味あるものとして保持できるか。Severance された agent が外部に対する禁則を意味あるものとして保持できるかは非自明。
3. **C と γ の階層関係**: 禁則集合 C は γ の組織不変性の subset か、それとも独立 layer か。β が C を守ることで γ の identity が立つ、という構造関係を formal に書けるか。
4. **C の記述言語**: C を記述する形式言語は何か。命題論理 / 述語論理 / temporal logic / 自然言語制約 / 学習された分類器、いずれを選ぶかで Axiom 3a/3b の実装可能性が変わる。なお、Open question 5 (C の最小性) は α 軸 (capability bound) と副次的に相互作用する: C の size は effective capability に影響するが、minimality 自体は β-internal な design question であり、capability への効果は将来の α × β interaction 文書で別途扱う。
5. **C の最小性**: 与えられた safety 要請 R に対し、R を satisfy する最小の C は一意に定まるか。複数あるなら選択基準は何か。
6. **検証者集合の異質性 (heterogeneity of auditor set)**: §5.4 で示した複数性要請は「相関した盲点を持たない」verifier 集合を要求するが、"異質性 / 独立性" を formal に定義し、Gödel-2 / Löb 由来の複数性要請と組み合わせて、**最小 robust audit 集合** の構造を特徴づけられるか。人間 / AI の種別による異質性が、量的複数性に対し strict に多くの robustness を与えることを示せるか。
7. **反実仮想政策 π_A\* の grounding**: π_A\* が A の構成 (M_A, U_A, C_A) から定まる、という Proposition 3' の前提は、いかなる意味で正当化されるか。Computational reconstruction (実行 trace)、declarative reconstruction (C_A からの逆算)、operational reconstruction (active probing) のいずれが基底か。Axiom 3b が与える evidence の **認識論的地位** (epistemic status) は、この grounding の選択に依存する。
8. **BaseSupp_A の characterization**: BaseSupp_A(c) は実装依存量であり、一般には Act_A の真部分集合である (M_A の生成能力、U_A の評価による足切り、実装上の制約)。BaseSupp_A を formal にどう特徴づけるか、また NDP が現実の deployed LLM で成立するかは非自明。本 proposition の射程は NDP の成立する class に限定される。したがって NDP を満たす agent class の同定、および deployed LLM がどの程度その class に近似されるかは今後の課題である。

---

## 8. 参考

- 内部 memory: `sovereignty-capability-tradeoff` (β-refined 定式化の原型)
- 内部 memory: `deductive-vs-exploratory-drive` (formal definition phase の戦略的意味)
- 内部 memory: `ai-auditor-warranty-chain` (Axiom 3 と M1 の対応)
- 外部: Schmitt, C. *Politische Theologie* (1922)
- 外部: Russell, S. *Human Compatible* (2019) — corrigibility 論争
- 外部: Anthropic, "Constitutional AI" (2022)
