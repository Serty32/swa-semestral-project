package cz.cvut.swa.bazaar.productservice.service

import com.fasterxml.jackson.databind.ObjectMapper
import cz.cvut.swa.bazaar.productservice.BaseTest
import cz.cvut.swa.bazaar.productservice.data.ReviewDTO
import cz.cvut.swa.bazaar.productservice.data.ReviewResponseDTO
import cz.cvut.swa.bazaar.productservice.timestamp
import org.junit.AfterClass
import org.junit.Assert
import org.junit.BeforeClass
import org.junit.Test
import org.mockserver.integration.ClientAndServer
import org.mockserver.integration.ClientAndServer.startClientAndServer
import org.mockserver.model.Header
import org.mockserver.model.HttpRequest.request
import org.mockserver.model.HttpResponse.response
import org.springframework.beans.factory.annotation.Autowired
import java.util.*

class ReviewServiceMockTest : BaseTest() {

    @Autowired
    private lateinit var reviewService: ReviewService

    @Autowired
    private lateinit var objectMapper: ObjectMapper

    companion object {
        lateinit var mockServer: ClientAndServer

        @JvmStatic
        @BeforeClass
        fun startServer() {
            mockServer = startClientAndServer(9090)
        }

        @JvmStatic
        @AfterClass
        fun stopServer() {
            mockServer.stop()
        }
    }

    @Test
    fun fetchProductReviews_shouldCallMockServer_shouldReturnReviews() {
        // given
        val productId = randomUuid()
        val reviewList = listOf(
                ReviewDTO(1L, 1L, "The product is absolutely perfect!", productId, timestamp(), 77),
                ReviewDTO(2L, 2L, "It's a scam!!", productId, timestamp(), 12)
        )

        val reviewResponse = ReviewResponseDTO(success = true, data = reviewList)

        mockServer.`when`(
                request()
                        .withMethod("GET")
                        .withPath("/reviews/product/$productId"))
                .respond(
                        response()
                                .withStatusCode(200)
                                .withHeaders(
                                        Header("Content-Type", "application/json; charset=utf-8"))
                                .withBody(objectMapper.writeValueAsString(reviewResponse))
                )

        // when
        val result = reviewService.fetchProductReviews(productId)

        // verify
        Assert.assertNotNull(result)
        Assert.assertEquals(2, result?.size)
        Assert.assertArrayEquals(reviewList.toTypedArray(), result?.toTypedArray()) // verify by deep array equals
    }

    private fun randomUuid() = UUID.randomUUID().toString()

}