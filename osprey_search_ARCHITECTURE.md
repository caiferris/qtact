# Osprey Search

**AI Search Engine**

A search engine which understands your users and provides best results from millions of products in the catalogue.

## How Does The Osprey Search Works?

The search flow starts with a `query` hitting the `Load balancer`.\
The request is then routed to an available back-end `server` (currently : `FastAPI` | to be replaced by : `Actix-Web`), which performs the `spell-correction` for the `query`.\
The `spell-correction` is cached for probable repeated erroneous queries.\
The `GuardRails` addresses the probable search filters present within the `query`, such as, 'Black shoes under 4000', tells the `Osprey` search to keep the price of product listings under 4000. Which means it generates payload filters for vector database.\
The `Stella Embedding Model` generates `vector` embeddings for the query.\
These filters and embeddings are then passed on to the `Qdrant Database` to fetch score ranked, payload filtered `PointIds` (Maximum 1,50,000).\

> These `PointIds` represents products with multiple `SKU`s (Stock Keeping Units).
> SKU represents that a certain product is available in different colour or sizes.

> [!NOTE]
> `PointIds` Data is Cached for frequent and fast retrieval

These `PointIds` are used to fetch results of the inventory from `ClickHouse` (A Columnar Database).\


User generates a query: -> We respond with the products for that specific query

Part  I - It is cached in the redis 
Query Goes to spell correction (Data Science/MLteam created a model which corrects the spelling).
Spell correction is cached 
GuardRails - Filters Within the query itself (Not the external user filters)
		(Uses redis as DB, keys are the keywords from the queries the values are properties of the keywords (products) - > Generate filters for qdrant
Goes to Stella Embedding Model -> generates vector embeddings for query
		Both are cached in separate keyspaces -  Generates the embedding vector.
Vector Embeddings goes to Qdrant -> Fetches top 1.5 L products (PointIds - multiple SKUs)
Exact Query -> If results less than 100 products. Substitute query -> uses substitute queries (addes keywords from guard rails) -> Compliment query -> expand search by removing some properties (more restrictive properties to ease the search) 
We get PointIds with Cosine scores related to the query search -> SKU are unique (PointIds will have multiple SKUs -> represents variants of the product) 
This pointID data is cached pointId and Scores -> Qdrant { brand categories (L1, L3 etc) -> comes from guardrails }
Part II
Clickhouse (good for aggregating data) stores products info (metadata) -> Returns least price SKU as top result.
It sorts on base of ordering
Problem - SQL query takes best case 1 second to complete - worst : 20-25 sec for 1.5 Lakh products
Can be reading a single granules into thousands rows 
Default granule size is 1024
Picks a whole granule for a single row.
Sol:
Sort by relevance (a different score comes from using a formula)
Window function -> to get lowest price SKUs
Responds with the final results provided from Clickhouse

Notes:
Filters from Guardrails are handled by Qdrant
And filters from user filters are handled by clickhouse
Every stage outputs a query for the next step
Qdrant takes input from both Guardrails and Stella
Size of each step cache
Worst case read latency -> 250ms 
Generally takes 10-20 ms
Facets comes from Qdrant
Cluster takes more time vs a single instance

Part2 Notes:
Takes 90% of time even with caching.
Clickhouse- Probabilistic distribution of data over granules

Optimisations tried:


Relevancy is updated in 4 hours -> 1-2 million SKUs are updated.
30 updates per second for inventory
5000 updates per second (in worst case) avg < 1000 = For prices.

Small joins are not optimal in Clickhouse
We have to use IN ops


