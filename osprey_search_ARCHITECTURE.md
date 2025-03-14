# Osprey Search

**AI Search Engine**

A search engine which understands your users and provides best results from millions of products in the catalogue.

> [!WARNING]
> The choice of tools is not concrete, can change in upcoming iterations.\
> **Qdrant** and **ClickHouse** can be replaced with something better.

## How Does The Osprey Search Works?

The search flow starts with a `query` hitting the `Load balancer`.\
The request is then routed to an available back-end `server` (currently : `FastAPI` | to be replaced by : `Actix-Web`), which performs the `spell-correction` for the `query`.\
The `spell-correction` is cached for probable repeated erroneous queries.\
The `GuardRails` addresses the probable search filters present within the `query`, such as, 'Black shoes under 4000', tells the `Osprey` search to keep the price of product listings under 4000. Which means it generates payload filters for vector database.\
The `Stella Embedding Model` generates `vector` embeddings for the query.\
These filters and embeddings are then passed on to the `Qdrant Database` to fetch score ranked, payload filtered `PointIds` (Maximum 1,50,000).

> [!NOTE]
> - These `PointIds` represents products with multiple `SKUs` (Stock Keeping Units).
> - SKU represents that a certain product is available in different colour or sizes.
> - `PointIds` Data is Cached for frequent and fast retrieval

These `PointIds` are used to fetch results of the inventory from `ClickHouse` (A Columnar Database).\
`ClickHouse` searches for `granules` which contain the `PointIds` and provides the metadata for the products result in *increasing order* of **SKUs** *price*.\
The `query` gets it's response as the resulted list of products provided by `ClickHouse`.

## Challenges

> [!CAUTION]
> The `Osprey` search engine can take more than **5 seconds** to process a request.

> [!NOTE]
> The search results take quite longer even with available optimizations possible within each layer.\
> The time taken by embedding generator model is known at the moment.

The `ClickHouse` query takes **90%** of the query processing time, which drags the overall average **QPS** throughput of the `Osprey` search.

## Architecture

The Architecture of `Osprey` search can be visually be presented as:

```mermaid
graph 
  query["Search A/B Service"]
  load_balancer["Load Balancer"]
  api_server["FastAPI RESTful service"]
  model_serving["Model Serving"]
  redis[(Redis Cache Cluster)]
  qdrant_vdb[(Qdrant Vector Database)]
  clickhouse_cdb[(ClickHouse DB - Product Metadata)]
  external_systems["External Systems"]
  kafka_producer["Kafka Producer"]
  kafka_topics[Kafka Topics]
  kafka_topic1["Price Engine (N.R.T)"]
  kafka_topic2["Inventory Engine (N.R.T)"]
  kafka_topic3["Relevancy Engine/PLP Rank\n (4 Times a Day)"]
  offline[Offline Processing]
  pipeline[Pipeline to Generate]
  product_embedding[Product Embeddings]
  top_qembedding[Top Query Embeddings]
  sftp[SFTP]
  model_training[Model Training]
  sdlr_cdump["`SDLR
Catalogue Dump
(Once Per Day)`"]
  rhos_obj_str_pea["`RHOS Object Storage
(Product Embeddings +
Attributes)`"]
  rhos_obj_str_tm["`RHOS Object Storage
(Trained Model)`"]
  cron["`DB Updating
K8s Cron`"]

  query --> load_balancer --> api_server
subgraph parent_subgraph["Osprey Search"]
  api_server -- 1a Look in Cache --> redis
  redis -- 1b Return data on Cache Hit --> api_server
  subgraph sub_graph_group1["Application Main K8s Cluster"]
    style sub_graph_group1 stroke: 5, 5
    api_server
    model_serving
  end
  subgraph qdrant_cluster["Qdrant K8s Cluster"]
    style qdrant_cluster stroke-dasharray: 5, 5
    qdrant_vdb -- Products Retrieval --> api_server
  end
  subgraph clickhouse_cluster["ClickHouse K8s Cluster"]
    style clickhouse_cluster stroke-dasharray: 5, 5
    clickhouse_cdb -- Facets And Retrieved Products Metadata --- api_server
  end
  kconsumer --> clickhouse_cluster
  cron --> qdrant_cluster
  cron --> clickhouse_cluster
end
subgraph external_systems
  style external_systems stroke-dasharray: 5, 5
  subgraph kafka_producer
    style kafka_producer stroke: 5, 5
    subgraph kafka_topics
      style kafka_topics stroke-dasharray: 5, 5
      kafka_topic1
      kafka_topic2
      kafka_topic3
    end
  end
end
kafka_producer --> kconsumer
subgraph offline
  subgraph pipeline
    product_embedding
    top_qembedding
  end
  sftp --> pipeline
  model_training
end
sdlr_cdump --> sftp
pipeline --> rhos_obj_str_pea
rhos_obj_str_pea --> cron
model_training --> rhos_obj_str_tm
rhos_obj_str_tm --> model_serving
```

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


